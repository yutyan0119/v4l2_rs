use libc::{open, O_NONBLOCK, O_RDWR};
use std::ffi::CString;
use std::io::{self, Write};
use std::os::unix::io::RawFd;
use v4l::v4l2;
use v4l2_sys_mit as v4l2_sys;

const V4L2_CAP_VIDEO_CAPTURE: u32 = 0x00000001; /* Is a video capture device */
const V4L2_CAP_STREAMING: u32 = 0x04000000; /* Streaming I/O ioctls */

struct Buffer {
    start: *mut std::ffi::c_void,
    length: u32,
}

fn main() -> io::Result<()> {
    let path: CString = CString::new("/dev/video0").unwrap();
    //let path: CString= CString::new("/dev/video0").unwrap().asptr();はダメ
    //即座に変換すると、すぐに破棄されてしまう
    //pointers do not have a lifetime; when calling `as_ptr` the `CString` will be deallocated at the end of the statement because nothing is referencing it as far as the type system is concerned
    let c_path: *const i8 = path.as_ptr();
    let fd: RawFd = unsafe { open(c_path, O_RDWR | O_NONBLOCK) };
    if fd == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    let mut v4l2_cap: v4l2_sys::v4l2_capability = unsafe { std::mem::zeroed() };
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_QUERYCAP,
            &mut v4l2_cap as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    if v4l2_cap.capabilities & V4L2_CAP_VIDEO_CAPTURE == 0 {
        println!("Error: V4L2_CAP_VIDEO_CAPTURE is not supported");
    }
    if v4l2_cap.capabilities & V4L2_CAP_STREAMING == 0 {
        println!("Error: V4L2_CAP_STREAMING is not supported");
    }
    let mut format: v4l2_sys::v4l2_format = unsafe { std::mem::zeroed() };
    format.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    format.fmt.pix.width = 1280;
    format.fmt.pix.height = 720;
    let m = b'M' as u32;
    let j = b'J' as u32;
    let p = b'P' as u32;
    let g = b'G' as u32;
    format.fmt.pix.pixelformat = (m << 24) | (j << 16) | (p << 8) | g;
    format.fmt.pix.field = v4l2_sys::v4l2_field_V4L2_FIELD_ANY;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_S_FMT,
            &mut format as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    format = unsafe { std::mem::zeroed() };
    format.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_G_FMT,
            &mut format as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }

    unsafe {
        println!("width: {}", format.fmt.pix.width);
        println!("height: {}", format.fmt.pix.height);
        println!("pixelformat: {}", format.fmt.pix.pixelformat);
    }

    let mut v4l2_reqbuf: v4l2_sys::v4l2_requestbuffers = unsafe { std::mem::zeroed() };
    v4l2_reqbuf.count = 3;
    v4l2_reqbuf.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    v4l2_reqbuf.memory = v4l2_sys::v4l2_memory_V4L2_MEMORY_MMAP;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_REQBUFS,
            &mut v4l2_reqbuf as *mut _ as *mut std::os::raw::c_void,
        )
    };
    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    println!("v4l2_reqbuf.count: {}", v4l2_reqbuf.count);

    let mut buffers: Vec<Buffer> = Vec::new();
    for index in 0..v4l2_reqbuf.count {
        let mut v4l2_buf: v4l2_sys::v4l2_buffer = unsafe { std::mem::zeroed() };
        v4l2_buf.index = index;
        v4l2_buf.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        v4l2_buf.memory = v4l2_sys::v4l2_memory_V4L2_MEMORY_MMAP;
        let ret: std::os::raw::c_int = unsafe {
            libc::ioctl(
                fd,
                v4l2::vidioc::VIDIOC_QUERYBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )
        };
        if ret == -1 {
            println!("Error: {}", io::Error::last_os_error());
        }
        unsafe {
            println!("v4l2_buf.length: {}", v4l2_buf.length);
            println!("v4l2_buf.m.offset: {}", v4l2_buf.m.offset);
        }
        let ptr: *mut std::os::raw::c_void = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                v4l2_buf.length as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                v4l2_buf.m.offset as libc::off_t,
            )
        };
        if ptr == libc::MAP_FAILED {
            println!("Error: {}", io::Error::last_os_error());
        }
        let buffer: Buffer = Buffer {
            start: ptr,
            length: v4l2_buf.length,
        };
        buffers.push(buffer);
    }

    for index in 0..v4l2_reqbuf.count {
        let mut v4l2_buf: v4l2_sys::v4l2_buffer = unsafe { std::mem::zeroed() };
        v4l2_buf.index = index;
        v4l2_buf.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
        v4l2_buf.memory = v4l2_sys::v4l2_memory_V4L2_MEMORY_MMAP;
        let ret: std::os::raw::c_int = unsafe {
            libc::ioctl(
                fd,
                v4l2::vidioc::VIDIOC_QBUF,
                &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
            )
        };
        if ret == -1 {
            println!("Error: {}", io::Error::last_os_error());
        }
    }

    let mut v4l2_buf_type: v4l2_sys::v4l2_buf_type =
        v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_STREAMON,
            &mut v4l2_buf_type as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }

    let mut v4l2_buf: v4l2_sys::v4l2_buffer = unsafe { std::mem::zeroed() };
    v4l2_buf.type_ = v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    v4l2_buf.memory = v4l2_sys::v4l2_memory_V4L2_MEMORY_MMAP;
    let ret = unsafe {
        libc::poll(
            &mut libc::pollfd {
                fd: fd,
                events: libc::POLLIN,
                revents: 0,
            },
            1,
            1000,
        )
    };
    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_DQBUF,
            &mut v4l2_buf as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }

    let data_slice: &[u8] = unsafe {
        std::slice::from_raw_parts::<u8>(
            buffers[v4l2_buf.index as usize].start as *mut u8,
            buffers[v4l2_buf.index as usize].length as usize,
        )
    };

    let mut file: std::fs::File = std::fs::File::create("test.jpg")?;
    match file.write_all(data_slice) {
        Ok(_) => {}
        Err(e) => {
            println!("error: {}", e);
        }
    };

    let mut v4l2_buf_type: v4l2_sys::v4l2_buf_type =
        v4l2_sys::v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
    let ret: std::os::raw::c_int = unsafe {
        libc::ioctl(
            fd,
            v4l2::vidioc::VIDIOC_STREAMOFF,
            &mut v4l2_buf_type as *mut _ as *mut std::os::raw::c_void,
        )
    };

    if ret == -1 {
        println!("Error: {}", io::Error::last_os_error());
    }
    Ok(())
}
