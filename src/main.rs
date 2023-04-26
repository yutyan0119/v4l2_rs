use std::io;
use std::io::Write;
use v4l::buffer::Type;
use v4l::capability::{Capabilities, Flags};
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;

fn main() -> io::Result<()> {
    let path: &str = "/dev/video0";
    let buffer_count: u32 = 3;
    let dev: Device = Device::with_path(path)?;
    let capability: Capabilities = dev.query_caps()?;
    if !capability.capabilities.contains(Flags::VIDEO_CAPTURE) {
        println!("The device does not support video capture");
        return Ok(());
    }
    if !capability.capabilities.contains(Flags::STREAMING) {
        println!("The device does not support video streaming");
        return Ok(());
    }
    let format_set: v4l::Format = v4l::Format::new(1920, 1080, v4l::FourCC::new(b"MJPG"));
    let format: v4l::Format = dev.set_format(&format_set)?;
    println!("Format: {:?}", format);
    let qctrls: Vec<v4l::control::Description>  = dev.query_controls()?;
    println!("Controls: {:?}", qctrls);
    for qctrl in qctrls {
        println!("{}", qctrl);
    }
    // let ctrl = v4l::Control::new(9963776, 0, 0, 0, 0);
    // dev.set_control(ctrl);
    let mut stream: MmapStream = MmapStream::with_buffers(& dev, Type::VideoCapture, buffer_count)?; 
    let (buf, meta) = stream.next()?;
    //write buffer to file
    let mut file = std::fs::File::create("test.jpg")?;
    file.write_all(&buf)?;
    println!("Buffer size: {}, seq: {}, timestamp: {}", buf.len(), meta.sequence, meta.timestamp);
    Ok(())
}
