#![allow(unused_assignments)]

use ndspy_sys;
use std::{
    ffi::CStr,
    fs, io, mem,
    os::raw::{c_char, c_int, c_uchar, c_void},
    path, ptr, slice,
};
use c_vec::CVec;
use png;

#[repr(C)]
#[derive(Debug)]
struct ImageData {
    data: Vec<u8>,
    offset: isize,
    width: u32,
    height: u32,
    channels: u32,
    file_name: String,
}

/// A utility function to get user parameters.
///
/// The template argument is the expected type of the resp. parameter.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the parameter we
///   are searching for
/// * `parameter_count` - Number of parameters
/// * `parameter`       - Array of `ndspy_sys::UserParameter` structs to
///   search
///
/// # Example
///
/// ```
/// let associate_alpha =
///     1 == get_parameter::<i32>("associatealpha", _parameter_count, _parameter).unwrap_or(0);
/// ```
pub fn get_parameter<T: Copy>(
    name: &str,
    parameter: &c_vec::CVec<ndspy_sys::UserParameter>,
) -> Option<T> {
    for p in parameter.iter() {
        if name == unsafe { CStr::from_ptr(p.name) }.to_str().unwrap() {
            let value_ptr = p.value as *const T;

            if value_ptr != ptr::null() {
                return Some(unsafe { *value_ptr });
            } else {
                // Value is missing, exit quietly.
                break;
            }
        }
    }

    None
}

#[no_mangle]
pub extern "C" fn DspyImageOpen(
    image_handle_ptr: *mut ndspy_sys::PtDspyImageHandle,
    _driver_name: *const c_char,
    output_filename: *const c_char,
    width: c_int,
    height: c_int,
    parameter_count: c_int,
    parameter: *mut ndspy_sys::UserParameter,
    format_count: c_int,
    format: *mut ndspy_sys::PtDspyDevFormat,
    flag_stuff: *mut ndspy_sys::PtFlagStuff,
) -> ndspy_sys::PtDspyError {
    if (image_handle_ptr == ptr::null_mut()) || (output_filename == ptr::null_mut()) {
        return ndspy_sys::PtDspyError_PkDspyErrorBadParams;
    }

    // Shadow C
    let mut format = // : Vec<ndspy_sys::PtDspyDevFormat> =
        unsafe { CVec::new(format, format_count as usize) }; //.into();

    // Ensure all channels are sent to us as 8bit integers.
    // This loops through each format (channel), r, g, b, a etc.
    // We also dump all formats to stderr.
    for i in 0..format.len() {
        format.get_mut(i).unwrap().type_ = ndspy_sys::PkDspyUnsigned8;
        eprintln!("{:?}", unsafe {
            CStr::from_ptr(format.get(i).unwrap().name)
        });
    }

    // Shadow C paramater array with wrapped version
    let parameter = unsafe { CVec::new(parameter, parameter_count as usize) };

    // Example use of get_parameter() helper.
    let _associate_alpha = 1 == get_parameter::<i32>("associatealpha", &parameter).unwrap_or(0);

    if output_filename != std::ptr::null() {
        let image = Box::new(ImageData {
            data: vec![0; (width * height * format_count) as usize],
            offset: 0,
            width: width as u32,
            height: height as u32,
            channels: format_count as u32,
            file_name: unsafe {
                CStr::from_ptr(output_filename)
                    .to_str()
                    .unwrap()
                    .to_string()
            },
        });

        // Get raw pointer to heap-allocated ImageData struct and pass
        // ownership to image_handle_ptr.
        unsafe {
            *image_handle_ptr = Box::into_raw(image) as *mut _;
        }

        unsafe {
            (*flag_stuff).flags |= ndspy_sys::PkDspyFlagsWantsScanLineOrder as i32;
        }

        ndspy_sys::PtDspyError_PkDspyErrorNone
    } else {
        // We're missing an output file name.
        ndspy_sys::PtDspyError_PkDspyErrorBadParams
    }
}

#[no_mangle]
pub extern "C" fn DspyImageQuery(
    image_handle: ndspy_sys::PtDspyImageHandle,
    query_type: ndspy_sys::PtDspyQueryType,
    data_len: c_int,
    mut data: *const c_void,
) -> ndspy_sys::PtDspyError {
    if (data == ptr::null_mut()) && (query_type != ndspy_sys::PtDspyQueryType_PkStopQuery) {
        return ndspy_sys::PtDspyError_PkDspyErrorBadParams;
    }

    // Looks like this is actually needed for a minimal implementation
    // as we never get called with the next two query types by 3Delight.
    // But we leave this code be – just in case. :]
    match query_type {
        ndspy_sys::PtDspyQueryType_PkSizeQuery => {
            let size_info = Box::new({
                if image_handle == ptr::null_mut() {
                    ndspy_sys::PtDspySizeInfo {
                        width: 1920,
                        height: 1080,
                        aspectRatio: 1.0,
                    }
                } else {
                    let image = unsafe { Box::from_raw(image_handle as *mut ImageData) };

                    ndspy_sys::PtDspySizeInfo {
                        width: image.width as u64,
                        height: image.height as u64,
                        aspectRatio: 1.0,
                    }
                }
            });

            debug_assert!(mem::size_of::<ndspy_sys::PtDspySizeInfo>() <= data_len as usize);

            // Transfer ownership of the size_query heap object to the
            // data pointer.
            data = Box::into_raw(size_info) as *mut _;
        }

        ndspy_sys::PtDspyQueryType_PkOverwriteQuery => {
            let overwrite_info = Box::new(ndspy_sys::PtDspyOverwriteInfo {
                overwrite: true as ndspy_sys::PtDspyUnsigned8,
                unused: 0,
            });

            // Transfer ownership of the size_query heap object to the
            // data pointer.
            data = Box::into_raw(overwrite_info) as *mut _;
        }

        _ => {
            return ndspy_sys::PtDspyError_PkDspyErrorUnsupported;
        }
    }

    ndspy_sys::PtDspyError_PkDspyErrorNone
}

#[no_mangle]
pub extern "C" fn DspyImageData(
    image_handle: ndspy_sys::PtDspyImageHandle,
    x_min: c_int,
    x_max_plus_one: c_int,
    y_min: c_int,
    y_max_plus_one: c_int,
    _entry_size: c_int,
    data: *const c_uchar,
) -> ndspy_sys::PtDspyError {
    let mut image = unsafe { Box::from_raw(image_handle as *mut ImageData) };

    if image_handle == ptr::null_mut() {
        return ndspy_sys::PtDspyError_PkDspyErrorBadParams;
    }

    let data_size =
        (image.channels as i32 * (x_max_plus_one - x_min) * (y_max_plus_one - y_min)) as usize;

    unsafe {
        ptr::copy_nonoverlapping(
            data,
            image.data.as_mut_ptr().offset(image.offset),
            data_size,
        );
    }

    image.offset += data_size as isize;

    // Important: we need to give up ownership of the boxed image or
    // else the compiler will free the memory on exiting this function.
    Box::into_raw(image);

    ndspy_sys::PtDspyError_PkDspyErrorNone
}

// PNG needs disassociated alpha
fn disassociate_alpha(image: &mut Box<ImageData>) -> &mut Box<ImageData> {
    for i in (0..image.data.len()).step_by(4) {
        let alpha = image.data[i + 3];
        if alpha != 0 {
            for c in i..i + 3 {
                let channel = image.data[c] as u32;
                // channel * 256 / alpha
                image.data[c] = (((channel << 8) - channel) / alpha as u32) as u8;
            }
        }
    }

    image
}

fn write_image(image: &Box<ImageData>) -> Result<(), png::EncodingError> {
    let path = path::Path::new(&image.file_name);
    match fs::File::create(path) {
        Ok(file) => {
            let writer = io::BufWriter::new(file);

            let mut encoder = png::Encoder::new(writer, image.width, image.height);
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;

            writer.write_image_data(unsafe {
                slice::from_raw_parts(image.data.as_ptr() as *const u8, image.data.len())
            })
        }
        Err(e) => {
            eprintln!("[r-display] Cannot open '{}' for writing.", path.display());
            Err(png::EncodingError::IoError(e))
        }
    }
}

#[no_mangle]
pub extern "C" fn DspyImageClose(
    image_handle: ndspy_sys::PtDspyImageHandle,
) -> ndspy_sys::PtDspyError {
    DspyImageDelayClose(image_handle)
}

#[no_mangle]
pub extern "C" fn DspyImageDelayClose(
    image_handle: ndspy_sys::PtDspyImageHandle,
) -> ndspy_sys::PtDspyError {
    let mut image = unsafe { &mut Box::from_raw(image_handle as *mut ImageData) };

    image = disassociate_alpha(image);

    match write_image(&image) {
        Ok(_) => ndspy_sys::PtDspyError_PkDspyErrorNone,
        Err(_) => ndspy_sys::PtDspyError_PkDspyErrorUndefined,
    }

    // image goes out of scope – this will free the memory
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
