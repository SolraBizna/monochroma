use monochroma::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = match args.len() {
        0 | 1 => "Sphinx of black quartz, judge my vow.",
        2 => &args[1],
        _ => {
            eprintln!("Usage: show_text \"some ASCII text\" < SomeFont.NFNT");
            std::process::exit(1)
        }
    };
    let plain =
        Font::read_mac_font(std::io::stdin()).expect("Couldn't load font");
    let height = (16
        + (plain.get_ascent() + plain.get_descent() + plain.get_leading()) * 8)
        as u32;
    let width = height * 2;
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let mut display = Display::new(&video, || {
        let mut ret = video.window("Bitmap Display", width, height);
        ret.resizable();
        ret
    })
    .unwrap();
    let bold = plain.make_bold();
    let italic = plain.make_italic();
    let bolditalic = bold.make_italic();
    let underline = plain.make_underline();
    let underlinebold = bold.make_underline();
    let underlineitalic = italic.make_underline();
    let underlinebolditalic = bolditalic.make_underline();
    let fonts_in_order = &[
        &plain,
        &bold,
        &italic,
        &bolditalic,
        &underline,
        &underlinebold,
        &underlineitalic,
        &underlinebolditalic,
    ];
    let mut bits: Bitmap = Bitmap::new(width, height);
    for (n, font) in fonts_in_order.iter().enumerate() {
        let y = 8
            + (font.get_ascent() + font.get_descent() + font.get_leading())
                * n as i32
            + font.get_ascent();
        bits.draw_text(
            ModeXor(()),
            None,
            16,
            y,
            &[*font],
            text.chars().map(|c| {
                if c > '\u{0080}' {
                    panic!("Non-ASCII character in argument text!")
                } else {
                    TextElement::DrawGlyph(c as u32 as u16)
                }
            }),
        );
    }
    #[cfg(feature = "netpbm")]
    {
        let _ = bits.write_ascii_pbm(
            std::io::stdout(),
            "Output of \"show_font\" example program for Monochroma",
        );
    }
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
