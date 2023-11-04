use monochroma::*;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    video.enable_screen_saver();
    let mut event_pump = sdl.event_pump().unwrap();
    let bits: Bitmap =
        Bitmap::read_netpbm(std::io::stdin()).expect("Couldn't load bitmap");
    let mut display = Display::new(&video, || {
        let mut ret = video.window(
            "Bitmap Display",
            bits.get_width(),
            bits.get_height(),
        );
        ret.resizable();
        ret
    })
    .unwrap();
    for event in event_pump.wait_iter() {
        use sdl2::event::Event;
        match event {
            Event::Quit { .. } | Event::KeyDown { .. } => break,
            Event::Window { win_event, .. } => {
                use sdl2::event::WindowEvent;
                match win_event {
                    WindowEvent::Exposed | WindowEvent::Shown => {
                        display
                            .update(
                                &bits,
                                1.0,
                                &[0.03, 0.03, 0.03, 1.0],
                                &[1.0, 1.0, 1.0, 1.0],
                                &[0.0, 0.0, 0.0, 1.0],
                                None,
                            )
                            .unwrap();
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}
