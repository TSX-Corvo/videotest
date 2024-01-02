use std::{process::exit, env};
use gstreamer as gst;
use gst::prelude::*;

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


fn on_pad_added(values: &[gst::glib::Value], next: &gst::Element) -> Option<gst::glib::Value> {
    // Assuming the values slice contains the pad
    let pad_value = values.get(1).unwrap();

    let pad = pad_value.get::<gst::Pad>().unwrap();
    
    // Wait for the pad to negotiate its capabilities

    let sinkpad = next.static_pad("sink").unwrap();

    if sinkpad.is_linked() {
        return None
    }
    
    if let Err(err) = pad.link(&sinkpad) {
        eprintln!("Failed to link pads: {}", err);
    }
          
    return None
}

fn main() {
    // env::set_var("RUST_BACKTRACE", "1");

    // Initialize GStreamer
    gst::init().expect("Failed to initialize GStreamer");

    let main_loop = gst::glib::MainLoop::new(None, false);

    // Create a pipeline
    let pipeline = gst::Pipeline::new(Some("camera-display"));
    
    let bus = pipeline.bus().unwrap();

    let _watch_id = bus.add_watch(bus_call).unwrap();

    // Create elements
    let source = gst::ElementFactory::make("filesrc", Option::Some("file-source")).unwrap();
    let decodebin = gst::ElementFactory::make("decodebin", Option::Some("dec")).unwrap();
    let videobalance = gst::ElementFactory::make("videobalance", Some("video-balance")).unwrap();
    let videoconvert = gst::ElementFactory::make("videoconvert", Some("video-converter")).unwrap();
    let autovideosink = gst::ElementFactory::make("autovideosink", Some("auto-video-sink")).unwrap();

    // Set camera source properties (device, resolution, etc.)
    let path = env::args().nth(1).expect("Filename not provided!");
    source.set_property("location", path);

    videobalance.set_property("saturation", 0.0);

    let elements = &[&source, &decodebin, &videobalance, &videoconvert, &autovideosink]; 

    // Add elements to the pipeline
    pipeline.add_many(elements).expect("Failed to add elements to the pipeline");

    // Link the elements
    source.link(&decodebin).expect("Failed to link source and dec");
    videobalance.link(&videoconvert).expect("Failed to link videoconvert and autovideosink");
    videoconvert.link(&autovideosink).expect("Failed to link videoconvert and autovideosink");

    decodebin.connect("pad-added", true, move | values | {
        return on_pad_added(values, &videobalance);
    });

    println!("Running...");

    // Set the pipeline to the playing state
    if let Err(err) = pipeline.set_state(gst::State::Playing) {
        eprintln!("Failed to play: {}", err);
    }
    
    // Create and run the main loop
    main_loop.run();

    println!("Returned, stopping playback");

    pipeline.set_state(gst::State::Null).unwrap();
}

