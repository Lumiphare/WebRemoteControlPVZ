extern crate ffmpeg_next as ffmpeg;

use std::io::Write;
use win_desktop_duplication::*;
use win_desktop_duplication::{tex_reader::*, devices::*};
use ffmpeg::{format, codec, util, Dictionary, log, encoder};
use ffmpeg::software::scaling;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()>{

    // this is required to be able to use desktop duplication api
    set_process_dpi_awareness();
    co_init();

    // select gpu and output you want to use.
    let adapter = AdapterFactory::new().get_adapter_by_idx(1).unwrap();
    let output = adapter.get_display_by_idx(0).unwrap();
    let width = output.get_current_display_mode().unwrap().width;
    let height = output.get_current_display_mode().unwrap().height;
    // get output duplication api
    let mut dupl = DesktopDuplicationApi::new(adapter, output).unwrap();

    // Optional: get TextureReader to read GPU textures into CPU.
    let (device, ctx) = dupl.get_device_and_ctx();
    let mut texture_reader = TextureReader::new(device, ctx);

    // 初始化ffmpeg
    ffmpeg::init().unwrap();
    log::set_level(log::Level::Info);

    let output_file = "output.mp4";
    let mut octx = format::output(&output_file)?;
    let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);

    let codec = encoder::find(codec::Id::H264).unwrap();
    let mut encoder = codec::context::Context::new_with_codec(codec)
        .encoder().video()?;

    encoder.set_height(height);
    encoder.set_width(width);
    encoder.set_time_base((1, 30));  // 表示每帧1/30秒
    encoder.set_format(format::Pixel::YUV420P);
    // if global_header {
    //     encoder.set_flags(codec::Flags::GLOBAL_HEADER);
    // }

    let mut ost = octx.add_stream(codec)?;
    ost.set_time_base((1,30));
    // ost.set_rate((30, 1));
    ost.set_parameters(&encoder);

    let mut dict = Dictionary::new();
    dict.set("preset", "medium");
    let mut encoder = encoder.open_with(dict).expect("TODO: panic message");

    let ost_idx = ost.index();
    let mut pic_data: Vec<u8> = vec![0; 0];
    octx.write_header()?;

    let stop_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_clone = stop_signal.clone();
    // 使用move移动外部变量所有权
    std::thread::spawn(move || {
        println!("Press 'q' to quit");
        let mut input = String::new();
        loop {
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim() == "q" {
                stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
            input.clear();
        }
    });
    let mut pts = 0;
    // let time_base = ffmpeg::Rational(1, 30);
    // let pts_increment = time_base.denominator() as i64 / time_base.numerator() as i64;

    // 放缩器
    let mut scaler = ffmpeg::software::scaling::Context::get(
        format::Pixel::BGRA, width, height,encoder.format(),
        width, height, scaling::Flags::BILINEAR).unwrap();

    loop {
        // this api send one frame per vsync. the frame also has cursor pre drawn
        let tex = dupl.acquire_next_vsync_frame().await;
        if let Ok(tex) = tex {
            texture_reader.get_data(&mut pic_data, &tex).unwrap();
            // use pic_data as necessary

            // 编码帧数据
            let mut frame = util::frame::Video::new(util::format::Pixel::BGRA, width, height);
            frame.data_mut(0).copy_from_slice(&pic_data);
            frame.set_kind(ffmpeg::picture::Type::None);

            if pic_data[0] == 0 && pic_data[1] == 0 && pic_data[2] == 0 {
                continue;
            }

            let mut scaled_frame = util::frame::Video::empty();
            scaler.run(&frame, &mut scaled_frame).unwrap();
            
            scaled_frame.set_pts(Some(pts));
            pts += 1;
            encoder.send_frame(&scaled_frame).unwrap();

            let mut packet = ffmpeg::codec::packet::Packet::empty();
            while encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(ost_idx);
                packet.write_interleaved(&mut octx).unwrap();
            }
        }
        if stop_signal.load(std::sync::atomic::Ordering::Relaxed) {
            encoder.send_eof()?;
            let mut packet = ffmpeg::codec::packet::Packet::empty();
            while encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(ost_idx);
                packet.write_interleaved(&mut octx).unwrap();
            }
            break;
        }
    }

    octx.write_trailer().unwrap();
    Ok(())
}