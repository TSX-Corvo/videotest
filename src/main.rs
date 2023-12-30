use std::env;
use std::process::exit;
use gstreamer as gst;
use gst::prelude::*;
use gst::glib::MainLoop;

fn bus_call(_bus: &gst::Bus, msg: &gst::Message) -> Continue {

    let view = msg.view();
    
    match view {
        gst::MessageView::Eos(_) => {
            println!("End of stream");
            exit(0);
        },
        gst::MessageView::Error(_) => {
            println!("Error: {:?}", view);
            exit(1);
        },
        _ => ()
    }

    return Continue(true);
}

fn on_pad_added(values: &[gst::glib::Value], decoder: &gst::Element) -> Option<gst::glib::Value> {
    // Assuming the values slice contains the pad
    if let Some(pad_value) = values.get(1) {
        if let Ok(pad) = pad_value.get::<gst::Pad>() {
            // Wait for the pad to negotiate its capabilities
            if let Some(caps) = pad.current_caps() {
                println!("Source Pad Caps: {:?}", caps);

                // Get the sink pad of the decoder
                if let Some(sinkpad) = decoder.static_pad("sink") {
                    // Wait for the sink pad to negotiate its capabilities
                  
                    // Now you can link the pads
                    if let Err(err) = pad.link(&sinkpad) {
                        eprintln!("Failed to link pads: {}", err);
                    }
                        
                    
                } else {
                    eprintln!("No sink pad found in decoder");
                }
            } else {
                eprintln!("Failed to get current caps for the source pad");
            }
        } else {
            eprintln!("Failed to get source pad from values");
        }
    } else {
        eprintln!("No source pad found in values");
    }

    // Return an Option<gst::glib::Value> if needed
    None
}

fn main() {
    // Initialize GStreamer
    gst::init().expect("Failed init");

    let main_loop = MainLoop::new(None, false);
    
    let pipeline = gst::Pipeline::new(Option::Some("audio-player"));
    let source = gst::ElementFactory::make("filesrc", Option::Some("file-source")).unwrap();
    let demuxer = gst::ElementFactory::make("oggdemux", Option::Some("ogg-demuxer")).unwrap();
    let decoder = gst::ElementFactory::make("vorbisdec", Option::Some("vorbis-decoder")).unwrap();

    let conv = gst::ElementFactory::make("audioconvert", Option::Some("converter")).unwrap();
    let sink = gst::ElementFactory::make("autoaudiosink", Option::Some("audio-output")).unwrap();

    let path = env::args().nth(1).expect("Filename not provided!");
    source.set_property("location", path);

    let bus = pipeline.bus().unwrap();

    let _watch_id = bus.add_watch(bus_call).unwrap();

    pipeline.add_many(&[&source, &demuxer, &decoder, &conv, &sink]).expect("Failed adding elements");

    source.link(&demuxer).expect("Fail link source demuxer");

    gst::Element::link_many(&[&decoder, &conv, &sink]).expect("Fail link decoder conv sink");

    demuxer.connect("pad-added", true, move | values | {
        // let decoder_clone = decoder.downgrade();
        return on_pad_added(values, &decoder);
    });

    println!("Running...");

    pipeline.set_state(gst::State::Playing).expect("No playea");

    main_loop.run();

    println!("Returned, stopping playback");

    pipeline.set_state(gst::State::Null).expect("No nullea");
}
