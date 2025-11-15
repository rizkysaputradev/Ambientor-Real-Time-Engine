//! Ambientor CLI — real-time player for evolving ambient scenes.

use ambientor_engine::graph::Engine;
use ambientor_engine::scenes::Scene;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Default)]
struct Args {
    list_devices: bool,
    device_name: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    duration_sec: Option<u64>,
    scene: Option<String>,
    gain: Option<f32>,
}

fn parse_args() -> Args {
    let mut a = Args::default();
    for s in std::env::args().skip(1) {
        if s == "--list-devices" { a.list_devices = true; continue; }
        if let Some(rest) = s.strip_prefix("--device=")       { a.device_name = Some(rest.to_string()); continue; }
        if let Some(rest) = s.strip_prefix("--sample-rate=")  { a.sample_rate = rest.parse().ok();     continue; }
        if let Some(rest) = s.strip_prefix("--channels=")     { a.channels    = rest.parse().ok();     continue; }
        if let Some(rest) = s.strip_prefix("--duration=")     { a.duration_sec= rest.parse().ok();     continue; }
        if let Some(rest) = s.strip_prefix("--scene=")        { a.scene       = Some(rest.to_string());continue; }
        if let Some(rest) = s.strip_prefix("--gain=")         { a.gain        = rest.parse().ok();     continue; }
        eprintln!("[warn] unknown arg: {s}");
    }
    a
}

fn list_output_devices() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    println!("Available output devices:");
    for dev in host.output_devices()? {
        println!("- {}", dev.name()?);
    }
    Ok(())
}

fn pick_device(args: &Args) -> Result<cpal::Device, Box<dyn Error>> {
    let host = cpal::default_host();
    if let Some(name) = &args.device_name {
        for d in host.output_devices()? {
            if d.name()? == *name { return Ok(d); }
        }
        return Err(format!("requested device not found: {name}").into());
    }
    host.default_output_device()
        .ok_or_else(|| "no default output device".into())
}

fn choose_config(
    device: &cpal::Device,
    req_sr: Option<u32>,
    req_ch: Option<u16>,
) -> Result<cpal::SupportedStreamConfig, Box<dyn Error>> {
    // If nothing requested, default is already concrete.
    if req_sr.is_none() && req_ch.is_none() {
        return Ok(device.default_output_config()?);
    }

    // Pick a SupportedStreamConfigRange first.
    let mut best: Option<(u64, cpal::SupportedStreamConfigRange)> = None;
    for range in device.supported_output_configs()? {
        let ch     = range.channels();
        let sr_min = range.min_sample_rate().0;
        let sr_max = range.max_sample_rate().0;

        let ch_pen = match req_ch { Some(c) => (i64::from(ch) - i64::from(c)).unsigned_abs() as u64, None => 0 };
        let sr_pen = match req_sr {
            Some(sr) => if (sr_min..=sr_max).contains(&sr) { 0 } else { sr_min.abs_diff(sr).min(sr_max.abs_diff(sr)) as u64 },
            None => 0,
        };

        let score = sr_pen.saturating_mul(1000) + ch_pen;
        if best.as_ref().map(|(s, _)| *s).map_or(true, |s| score < s) {
            best = Some((score, range));
        }
    }

    let (_, range) = best.ok_or_else(|| "no supported output configs".to_string())?;

    // Choose a concrete sample rate and convert the range into a concrete config.
    let pick_sr = match req_sr {
        Some(sr) => {
            let lo = range.min_sample_rate().0;
            let hi = range.max_sample_rate().0;
            cpal::SampleRate(sr.clamp(lo, hi))
        }
        None => range.max_sample_rate(),
    };

    Ok(range.with_sample_rate(pick_sr))
}

fn make_scene(name: Option<&str>, sr: f32) -> Scene {
    match name.unwrap_or("slow-drone").to_ascii_lowercase().as_str() {
        "slow-drone" | _ => Scene::slow_drone(sr),
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    cfg: &cpal::StreamConfig,
    mut engine: Engine<Scene>,
    gain: f32,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, Box<dyn Error>>
where
    T: cpal::Sample + cpal::FromSample<f32> + cpal::SizedSample + Send + 'static,
{
    let sr = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;

    // ~1 second meter at requested rate
    let meter_interval = (cfg.sample_rate.0).max(1) as usize;
    let mut meter_count: usize = 0;
    let mut meter_peak: f32 = 0.0;

    let stream = device.build_output_stream(
        cfg,
        move |output: &mut [T], _| {
            for frame in output.chunks_mut(channels) {
                let mut s = engine.next(sr) * gain;
                if s >  1.0 { s =  1.0; }
                if s < -1.0 { s = -1.0; }

                let v: T = T::from_sample(s);
                for ch in frame.iter_mut() { *ch = v; }

                // naive peak meter
                let a = if s >= 0.0 { s } else { -s };
                if a > meter_peak { meter_peak = a; }
                meter_count += 1;
                if meter_count >= meter_interval {
                    eprintln!("[meter] peak ~ {:.3}", meter_peak);
                    meter_peak = 0.0;
                    meter_count = 0;
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    if args.list_devices {
        list_output_devices()?;
        return Ok(());
    }

    println!("ambientor-cli — real-time ambient player\n");

    let device  = pick_device(&args)?;
    let sup_cfg = choose_config(&device, args.sample_rate, args.channels)?;
    let sample_format = sup_cfg.sample_format();
    let mut cfg = sup_cfg.config();

    if let Some(sr) = args.sample_rate { cfg.sample_rate = cpal::SampleRate(sr); }
    if let Some(ch) = args.channels    { cfg.channels    = ch; }

    let sr_f32 = cfg.sample_rate.0 as f32;
    let engine = Engine::new(make_scene(args.scene.as_deref(), sr_f32));
    let gain   = args.gain.unwrap_or(0.35);

    println!("Using device: {}", device.name()?);
    println!("Stream config: {:?} (sample_format: {:?})", cfg, sample_format);
    println!("Scene: {}  | Gain: {:.2}", args.scene.as_deref().unwrap_or("slow-drone"), gain);
    if let Some(d) = args.duration_sec { println!("Auto-stop after {d} seconds"); }
    println!("Press Ctrl+C to stop…\n");

    // Provide the missing error callback and pass it in.
    let err_fn = |e: cpal::StreamError| eprintln!("[cpal] stream error: {e}");

    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &cfg, engine, gain, err_fn)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &cfg, engine, gain, err_fn)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &cfg, engine, gain, err_fn)?,
        other => return Err(format!("unsupported device sample format: {other:?}").into()),
    };

    stream.play()?;

    if let Some(d) = args.duration_sec {
        std::thread::sleep(Duration::from_secs(d));
        return Ok(());
    }

    loop { std::thread::sleep(Duration::from_millis(500)); }
}
