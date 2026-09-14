mod lake_geometry;
mod river_geometry;

use lake_geometry::{TAUPO_ASPECT, TAUPO_SHORE};
use leptos::{html, mount::mount_to_body, prelude::*};
use river_geometry::WAIKATO_RIVER;
use std::{cell::RefCell, f64::consts::TAU, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, PointerEvent};

const EVENTBRITE_URL: &str =
    "https://www.eventbrite.co.nz/d/new-zealand--hamilton/waikato-business/";

#[derive(Clone)]
struct Star {
    x: f64,
    y: f64,
    radius: f64,
    alpha: f64,
    speed: f64,
    phase: f64,
}

struct SkyState {
    width: f64,
    height: f64,
    pointer_x: f64,
    pointer_y: f64,
    stars: Vec<Star>,
    reduced_motion: bool,
}

impl SkyState {
    fn new() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            pointer_x: 0.0,
            pointer_y: 0.0,
            stars: Vec::new(),
            reduced_motion: reduced_motion(),
        }
    }
}

fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .is_some_and(|query| query.matches())
}

fn noise(seed: u32) -> f64 {
    let value = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (value % 10_000) as f64 / 10_000.0
}

fn rebuild_stars(state: &mut SkyState) {
    let count = ((state.width * state.height / 8_400.0) as usize).clamp(58, 190);
    state.stars = (0..count)
        .map(|index| {
            let seed = index as u32 + 17;
            Star {
                x: noise(seed) * state.width,
                y: noise(seed.wrapping_mul(7)) * state.height * 0.82,
                radius: if index % 23 == 0 {
                    1.5
                } else {
                    noise(seed.wrapping_mul(11)) * 0.72 + 0.16
                },
                alpha: noise(seed.wrapping_mul(19)) * 0.62 + 0.14,
                speed: noise(seed.wrapping_mul(29)) * 0.0018 + 0.0005,
                phase: noise(seed.wrapping_mul(31)) * TAU,
            }
        })
        .collect();
}

fn draw_matariki(
    context: &CanvasRenderingContext2d,
    state: &SkyState,
    time: f64,
    drift_x: f64,
    drift_y: f64,
) {
    // Seven visible whetu foreground the Waikato-Tainui view of Matariki.
    let pattern = [
        (-0.34, -0.04, 1.0),
        (-0.17, -0.23, 0.74),
        (-0.06, 0.04, 1.18),
        (0.14, -0.12, 0.82),
        (0.33, -0.01, 0.68),
        (-0.16, 0.28, 0.72),
        (0.18, 0.22, 0.94),
    ];
    let scale = (state.width * 0.095).clamp(58.0, 118.0);
    let centre_x = state.width * if state.width < 700.0 { 0.73 } else { 0.78 } + drift_x;
    let centre_y = state.height * if state.width < 700.0 { 0.2 } else { 0.23 } + drift_y;

    context.begin_path();
    context.set_line_width(0.55);
    context.set_stroke_style_str("rgba(241,238,229,0.18)");
    let _ = context.ellipse(
        centre_x,
        centre_y,
        scale * 0.62,
        scale * 0.48,
        -0.12,
        0.0,
        TAU,
    );
    context.stroke();

    for (index, (x, y, size)) in pattern.iter().enumerate() {
        let pulse = if state.reduced_motion {
            1.0
        } else {
            1.0 + (time * 0.0015 + index as f64 * 0.88).sin() * 0.16
        };
        let star_x = centre_x + x * scale;
        let star_y = centre_y + y * scale;
        let radius = (2.1 + size * 1.45) * pulse;

        context.begin_path();
        context.set_fill_style_str("rgba(214,64,50,0.12)");
        let _ = context.arc(star_x, star_y, radius * 4.5, 0.0, TAU);
        context.fill();
        context.begin_path();
        context.set_fill_style_str("rgba(244,242,233,0.98)");
        let _ = context.arc(star_x, star_y, radius, 0.0, TAU);
        context.fill();
        context.begin_path();
        context.set_line_width(0.55);
        context.set_stroke_style_str("rgba(214,64,50,0.78)");
        context.move_to(star_x - radius * 2.6, star_y);
        context.line_to(star_x + radius * 2.6, star_y);
        context.move_to(star_x, star_y - radius * 2.6);
        context.line_to(star_x, star_y + radius * 2.6);
        context.stroke();
    }
}

fn draw_taupo(context: &CanvasRenderingContext2d, state: &SkyState) {
    let maximum_width = state.width * if state.width < 700.0 { 0.9 } else { 0.52 };
    let lake_width = maximum_width.min(state.height * 0.78 * TAUPO_ASPECT);
    let lake_height = lake_width / TAUPO_ASPECT;
    let river_source_x = river_x(state.width, WAIKATO_RIVER[0].1, 0.0);
    let origin_x = river_source_x - TAUPO_SHORE[0].0 * lake_width;
    let origin_y = state.height * 0.12;

    context.begin_path();
    for (index, &(x, y)) in TAUPO_SHORE.iter().enumerate() {
        let px = origin_x + x * lake_width;
        let py = origin_y + y * lake_height;
        if index == 0 {
            context.move_to(px, py);
        } else {
            context.line_to(px, py);
        }
    }
    context.close_path();
    context.set_fill_style_str("rgba(214,64,50,0.075)");
    context.fill();
    context.set_line_width(1.15);
    context.set_stroke_style_str("rgba(214,64,50,0.48)");
    context.stroke();
}

fn draw_sky(
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    state: &mut SkyState,
    time: f64,
) {
    let width = canvas.client_width() as f64;
    let height = canvas.client_height() as f64;
    let ratio = web_sys::window()
        .map(|window| window.device_pixel_ratio().min(2.0))
        .unwrap_or(1.0);

    if width != state.width || height != state.height {
        state.width = width;
        state.height = height;
        canvas.set_width((width * ratio) as u32);
        canvas.set_height((height * ratio) as u32);
        let _ = context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
        rebuild_stars(state);
    }

    context.clear_rect(0.0, 0.0, width, height);
    let drift_x = state.pointer_x * 13.0;
    let drift_y = state.pointer_y * 9.0;

    draw_taupo(context, state);

    for star in &state.stars {
        let shimmer = if state.reduced_motion {
            1.0
        } else {
            0.7 + (time * star.speed + star.phase).sin() * 0.3
        };
        context.begin_path();
        context.set_fill_style_str(&format!("rgba(238,239,226,{})", star.alpha * shimmer));
        let _ = context.arc(
            star.x + drift_x * star.radius,
            star.y + drift_y * star.radius,
            star.radius,
            0.0,
            TAU,
        );
        context.fill();
    }

    draw_matariki(context, state, time, drift_x, drift_y);

    let signal_y = height * 0.72 + drift_y;
    let glow = context.create_linear_gradient(0.0, signal_y, width, signal_y);
    let _ = glow.add_color_stop(0.0, "rgba(241,238,229,0)");
    let _ = glow.add_color_stop(0.44, "rgba(241,238,229,0.03)");
    let _ = glow.add_color_stop(0.52, "rgba(214,64,50,0.36)");
    let _ = glow.add_color_stop(1.0, "rgba(241,238,229,0)");
    context.set_fill_style_canvas_gradient(&glow);
    context.fill_rect(0.0, signal_y, width, 1.0);

    if !state.reduced_motion {
        let sweep_x = ((time * 0.035) % (width + 300.0)) - 150.0;
        let sweep = context.create_linear_gradient(sweep_x - 80.0, 0.0, sweep_x + 80.0, 0.0);
        let _ = sweep.add_color_stop(0.0, "rgba(241,238,229,0)");
        let _ = sweep.add_color_stop(0.5, "rgba(241,238,229,0.08)");
        let _ = sweep.add_color_stop(1.0, "rgba(241,238,229,0)");
        context.set_fill_style_canvas_gradient(&sweep);
        context.fill_rect(sweep_x - 80.0, 0.0, 160.0, height);
    }
}

fn start_signal_field(canvas: HtmlCanvasElement) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(raw_context)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(context) = raw_context.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let state = Rc::new(RefCell::new(SkyState::new()));
    let pointer_state = Rc::clone(&state);
    let pointer = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
        let mut state = pointer_state.borrow_mut();
        if state.width > 0.0 && state.height > 0.0 {
            state.pointer_x = event.client_x() as f64 / state.width - 0.5;
            state.pointer_y = event.client_y() as f64 / state.height - 0.5;
        }
    });
    let _ =
        window.add_event_listener_with_callback("pointermove", pointer.as_ref().unchecked_ref());
    pointer.forget();

    let animation: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let animation_handle = Rc::clone(&animation);
    let animation_window = window.clone();
    let animation_canvas = canvas.clone();
    let animation_state = Rc::clone(&state);

    *animation.borrow_mut() = Some(Closure::new(move |time: f64| {
        draw_sky(
            &animation_canvas,
            &context,
            &mut animation_state.borrow_mut(),
            time,
        );
        if !animation_state.borrow().reduced_motion {
            if let Some(callback) = animation_handle.borrow().as_ref() {
                let _ = animation_window.request_animation_frame(callback.as_ref().unchecked_ref());
            }
        }
    }));

    if let Some(callback) = animation.borrow().as_ref() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    };
}

fn river_x(width: f64, normalized_x: f64, progress: f64) -> f64 {
    let span = if width < 700.0 {
        width * 0.28
    } else {
        (width * 0.48).min(650.0)
    };
    // The final 19 km turn sharply west before Te Pūaha o Waikato meets the sea.
    // Smoothstep keeps the geographic line intact upstream while making that
    // coastal hook legible on a tall, narrow page.
    let lower_river = ((progress - 0.955) / 0.045).clamp(0.0, 1.0);
    let lower_river = lower_river * lower_river * (3.0 - 2.0 * lower_river);
    let hook_scale = if width < 700.0 { 0.65 } else { 0.48 };
    let western_hook = lower_river * span * hook_scale;
    width * 0.5 + (normalized_x - 0.5) * span - western_hook
}

fn trace_river(
    context: &CanvasRenderingContext2d,
    width: f64,
    document_height: f64,
    scroll_y: f64,
    source_y: f64,
) {
    context.begin_path();
    for (index, &(progress, x)) in WAIKATO_RIVER.iter().enumerate() {
        let px = river_x(width, x, progress);
        let py = source_y + progress * (document_height - source_y) - scroll_y;
        if index == 0 {
            context.move_to(px, py);
        } else {
            context.line_to(px, py);
        }
    }
}

fn draw_river(canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d, time: f64) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(root) = document.document_element() else {
        return;
    };
    let width = canvas.client_width() as f64;
    let height = canvas.client_height() as f64;
    let ratio = window.device_pixel_ratio().min(2.0);
    let pixel_width = (width * ratio) as u32;
    let pixel_height = (height * ratio) as u32;
    if canvas.width() != pixel_width || canvas.height() != pixel_height {
        canvas.set_width(pixel_width);
        canvas.set_height(pixel_height);
    }
    let _ = context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);
    context.clear_rect(0.0, 0.0, width, height);

    let scroll_y = window.scroll_y().unwrap_or(0.0);
    let document_height = (root.scroll_height() as f64).max(height);
    let maximum_lake_width = width * if width < 700.0 { 0.9 } else { 0.52 };
    let lake_width = maximum_lake_width.min(height * 0.78 * TAUPO_ASPECT);
    let lake_height = lake_width / TAUPO_ASPECT;
    let source_y = height * 0.12 + TAUPO_SHORE[0].1 * lake_height;
    context.set_line_cap("round");
    context.set_line_join("round");

    trace_river(context, width, document_height, scroll_y, source_y);
    context.set_stroke_style_str("rgba(169,48,39,0.94)");
    context.set_line_width(if width < 700.0 { 34.0 } else { 54.0 });
    context.stroke();

    trace_river(context, width, document_height, scroll_y, source_y);
    context.set_stroke_style_str("rgba(8,9,8,0.98)");
    context.set_line_width(if width < 700.0 { 25.0 } else { 42.0 });
    context.stroke();

    trace_river(context, width, document_height, scroll_y, source_y);
    context.set_stroke_style_str("rgba(241,238,229,0.58)");
    context.set_line_width(1.35);
    context.stroke();

    let moving = if reduced_motion() {
        0.0
    } else {
        time * 0.000018
    };
    for index in 0..7 {
        let progress = (moving + index as f64 / 7.0) % 1.0;
        let point_index = WAIKATO_RIVER.partition_point(|point| point.0 < progress);
        if let Some(&(_, x)) = WAIKATO_RIVER.get(point_index.min(WAIKATO_RIVER.len() - 1)) {
            let py = source_y + progress * (document_height - source_y) - scroll_y;
            if py > -20.0 && py < height + 20.0 {
                context.begin_path();
                context.set_fill_style_str("rgba(241,238,229,0.95)");
                let _ = context.arc(river_x(width, x, progress), py, 3.2, 0.0, TAU);
                context.fill();
            }
        }
    }

    let markers = [
        0.003, 0.021, 0.275, 0.368, 0.574, 0.600, 0.681, 0.740, 0.790, 0.837, 0.901, 0.914, 0.984,
    ];
    for (index, progress) in markers.iter().enumerate() {
        let point_index = WAIKATO_RIVER.partition_point(|point| point.0 < *progress);
        if let Some(&(_, x)) = WAIKATO_RIVER.get(point_index.min(WAIKATO_RIVER.len() - 1)) {
            let px = river_x(width, x, *progress);
            let py = source_y + progress * (document_height - source_y) - scroll_y;
            if py > -50.0 && py < height + 50.0 {
                let pulse = if reduced_motion() {
                    0.0
                } else {
                    (time * 0.0014 + index as f64).sin() * 3.0
                };
                context.begin_path();
                context.set_line_width(1.0);
                context.set_stroke_style_str("rgba(241,238,229,0.82)");
                let marker_radius = if index == 6 { 19.0 } else { 10.0 };
                let _ = context.arc(px, py, marker_radius + pulse, 0.18, TAU - 0.44);
                context.stroke();
                context.begin_path();
                context.set_fill_style_str("rgba(214,64,50,0.96)");
                let _ = context.arc(px, py, 2.5, 0.0, TAU);
                context.fill();
            }
        }
    }
}

fn start_river_field(canvas: HtmlCanvasElement) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(raw_context)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(context) = raw_context.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let animation: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let animation_handle = Rc::clone(&animation);
    let animation_window = window.clone();
    let animation_canvas = canvas.clone();
    *animation.borrow_mut() = Some(Closure::new(move |time: f64| {
        draw_river(&animation_canvas, &context, time);
        if let Some(callback) = animation_handle.borrow().as_ref() {
            let _ = animation_window.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }));
    if let Some(callback) = animation.borrow().as_ref() {
        let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    };
}

#[component]
fn PartnerLogo(href: &'static str, src: &'static str, alt: &'static str) -> impl IntoView {
    view! {
        <a class="partner-logo" href=href target="_blank" rel="noreferrer">
            <img src=src alt=alt loading="lazy" decoding="async" />
        </a>
    }
}

#[component]
fn App() -> impl IntoView {
    let sky_ref = NodeRef::<html::Canvas>::new();
    let river_ref = NodeRef::<html::Canvas>::new();
    let selected_stage = RwSignal::new(0usize);
    let stage_prompts = [
        (
            "An idea",
            "Bring the rough thought. Find the person who asks the useful question.",
        ),
        (
            "First sale",
            "Meet customers. Test whether value really changes hands.",
        ),
        (
            "Growing",
            "Share the knot you cannot untie alone. Someone here may have the thread.",
        ),
        (
            "Experienced",
            "Bring a lesson, an introduction or a door you can hold open.",
        ),
        (
            "Capital",
            "Meet builders early. Listen for conviction before the pitch is polished.",
        ),
    ];

    Effect::new(move |_| {
        if let Some(canvas) = sky_ref.get() {
            start_signal_field(canvas);
        }
        if let Some(canvas) = river_ref.get() {
            start_river_field(canvas);
        }
    });

    view! {
        <canvas node_ref=river_ref id="river-field" aria-hidden="true"></canvas>

        <header class="masthead">
            <a class="masthead__brand" href="#top" aria-label="Waikato Entrepreneur Meetup home">
                <span>"WAIKATO ENTREPRENEUR"</span><strong>"/ BIZ MEETUP"</strong>
            </a>
            <span class="masthead__signal" aria-hidden="true"></span>
            <a class="signup" href=EVENTBRITE_URL target="_blank" rel="noreferrer">
                <span>"Oct 15th / Signup"</span>
                <b aria-hidden="true">"↗"</b>
            </a>
        </header>

        <main>
            <nav class="river-towns" aria-label="Places along the Waikato River">
                <a class="river-town river-town--east" style="--route: 1.0" href="#top"><span>"Taupō"</span><small>"Lake / 357 m"</small></a>
                <a class="river-town river-town--east" style="--route: 2.1" href="#source"><span>"Huka Falls"</span><small>"9 km"</small></a>
                <a class="river-town river-town--east" style="--route: 27.5" href="#east-bank"><span>"Ātiamuri"</span><small>"117 km"</small></a>
                <a class="river-town river-town--west" style="--route: 36.8" href="#west-bank"><span>"Mangakino"</span><small>"156 km"</small></a>
                <a class="river-town river-town--east" style="--route: 57.4" href="#exchange"><span>"Karāpiro"</span><small>"244 km"</small></a>
                <a class="river-town river-town--west" style="--route: 60.0" href="#next-bend"><span>"Cambridge"</span><small>"255 km"</small></a>
                <a class="river-town river-town--major river-town--east" style="--route: 68.1" href="#kirikiriroa"><span>"Kirikiriroa"</span><strong>"Hamilton"</strong><small>"289 km / the meeting place"</small></a>
                <a class="river-town river-town--west" style="--route: 74.0" href="#hosts"><span>"Ngāruawāhia"</span><small>"315 km"</small></a>
                <a class="river-town river-town--east" style="--route: 79.0" href="#partners"><span>"Rāhui Pōkeka"</span><small>"Huntly / 335 km"</small></a>
                <a class="river-town river-town--west" style="--route: 83.7" href="#partners"><span>"Rangiriri"</span><small>"356 km"</small></a>
                <a class="river-town river-town--east" style="--route: 90.1" href="#port"><span>"Meremere"</span><small>"383 km"</small></a>
                <a class="river-town river-town--west" style="--route: 91.4" href="#port"><span>"Mercer"</span><small>"388 km"</small></a>
                <a class="river-town river-town--east" style="--route: 98.4" href="#port"><span>"Te Pūaha o Waikato"</span><small>"Port Waikato / 425 km / 0 m"</small></a>
            </nav>

            <section class="scene" id="top" aria-labelledby="page-title">
                <canvas node_ref=sky_ref id="signal-field" aria-hidden="true"></canvas>
                <div class="grain" aria-hidden="true"></div>
                <div class="scanlines" aria-hidden="true"></div>
                <div class="hero">
                    <p class="eyebrow">"Kirikiriroa, Aotearoa "<span>"/"</span>" First gathering incoming"</p>
                    <h1 id="page-title"><span>"Entrepreneur"</span>" "<em>"Meetup"</em></h1>
                    <p class="hero__line">"Ideas move when people do."</p>
                    <p class="hero__intro">"A room for current and future entrepreneurs. Meet people. Trade questions. Find the next bend."</p>
                </div>

                <div class="welcome-band" aria-label="Welcomes from our community">
                    <div class="welcome-band__track">
                        <span>"Kia ora"</span><i></i><span>"Ngā mihi"</span><i></i>
                        <span>"Talofa"</span><i></i><span>"Mālō e lelei"</span><i></i>
                        <span>"नमस्ते · Namaste"</span><i></i><span>"你好 · Nǐ hǎo"</span><i></i>
                        <span>"Kia orana"</span><i></i><span>"Bula vinaka"</span><i></i>
                        <span>"Kia ora"</span><i></i><span>"Ngā mihi"</span><i></i>
                        <span>"Talofa"</span><i></i><span>"Mālō e lelei"</span><i></i>
                        <span>"नमस्ते · Namaste"</span><i></i><span>"你好 · Nǐ hǎo"</span><i></i>
                        <span>"Kia orana"</span><i></i><span>"Bula vinaka"</span><i></i>
                    </div>
                </div>
                <div class="source-label"><span>"LAKE TAUPŌ"</span><b>"357 M"</b></div>
            </section>

            <section class="event-facts" aria-labelledby="event-facts-title">
                <div class="event-facts__lead">
                    <p class="bend__number">"THE FIRST GATHERING"</p>
                    <h2 id="event-facts-title">"A meetup with a job to do."</h2>
                    <p>"Come as you are. Leave with one honest conversation and one useful introduction."</p>
                </div>
                <dl class="event-facts__grid">
                    <div><dt>"When"</dt><dd>"Thursday 15 October 2026"</dd></div>
                    <div><dt>"Where"</dt><dd>"Kirikiriroa / venue revealed with registration"</dd></div>
                    <div><dt>"Who"</dt><dd>"Founders, future founders, investors and practical supporters"</dd></div>
                    <div><dt>"Why"</dt><dd>"Meet beyond your usual circle and help an idea move"</dd></div>
                </dl>
            </section>

            <section class="river-story" aria-label="The Waikato Entrepreneur Meetup journey">
                <article class="bend bend--source" id="source">
                    <div class="bend__copy bank--west">
                        <p class="bend__number">"01 / SOURCE"</p>
                        <p class="kicker">"Altered Capital"</p>
                        <h2>"Back the person before the polish."</h2>
                        <p>"Every venture begins upstream: a hunch, a problem, a person prepared to start. Capital matters. So do courage, candour and the first useful introduction."</p>
                        <a class="text-link" href="https://alteredcapital.com" target="_blank" rel="noreferrer">"Visit Altered Capital "<span>"↗"</span></a>
                    </div>
                    <figure class="bend__media bank--east media--landscape">
                        <img src="assets/hiko-people-two.webp" alt="A presenter sharing ideas with a Waikato audience" loading="lazy" decoding="async" />
                        <figcaption>"Start with what you know. Leave with who you met."</figcaption>
                    </figure>
                </article>

                <article class="bend bend--east" id="east-bank">
                    <div class="bend__copy bank--east">
                        <p class="bend__number">"02 / EAST BANK"</p>
                        <p class="kicker">"Hiko Hub (University of Waikato)"</p>
                        <h2>"Knowledge meets momentum."</h2>
                        <p>"Researchers, engineers, computer scientists, students and founders belong at the same table. Bring the thing you are learning and the thing you cannot yet solve."</p>
                        <p class="welcome-call">"Students and new grads: you belong in the room."</p>
                    </div>
                    <figure class="bend__media bank--west">
                        <img src="assets/hiko-hub.jpg" alt="Hiko Hub at the University of Waikato" loading="lazy" decoding="async" />
                        <figcaption>"Hiko Hub / Te Whare Ohaoha"</figcaption>
                    </figure>
                </article>

                <article class="bend bend--west" id="west-bank">
                    <div class="bend__copy bank--west">
                        <p class="bend__number">"03 / WEST BANK"</p>
                        <p class="kicker">"Soda (Wintec)"</p>
                        <h2>"Practice makes possibility real."</h2>
                        <p>"Get beyond the comfortable conversation. Meet operators, makers and founders who can test the idea against real work, real customers and real constraints."</p>
                    </div>
                    <figure class="bend__media bank--east">
                        <img src="assets/women-growth-lab.webp" alt="Women founders working together in Soda's Growth Lab" loading="lazy" decoding="async" />
                        <figcaption>"More voices. Better ventures."</figcaption>
                    </figure>
                </article>

                <section class="confluence" aria-labelledby="confluence-title">
                    <p class="bend__number">"04 / CONFLUENCE"</p>
                    <h2 id="confluence-title">"One river. Two banks. A shared room."</h2>
                    <p>"A jointly held and sponsored event, alternating between Hiko Hub at the University of Waikato and Soda at Wintec."</p>
                    <div class="bank-key" aria-label="Partner banks">
                        <span><i></i>"East / Hiko Hub (University of Waikato)"</span>
                        <span><i></i>"West / Soda (Wintec)"</span>
                    </div>
                </section>

                <article class="bend bend--exchange" id="exchange">
                    <div class="bend__copy bank--east">
                        <p class="bend__number">"05 / EXCHANGE"</p>
                        <p class="kicker">"Meet the market"</p>
                        <h2>"Value has to change hands."</h2>
                        <p>"A business exchanges a product or service for money. That means meeting the people with the problem, the budget, the experience or the capital, then listening closely."</p>
                        <blockquote>"Customers before assumptions. Conversations before theatre."</blockquote>
                    </div>
                    <figure class="bend__media bank--west">
                        <img src="assets/business-conversation.jpg" alt="Two women in business talking face to face" loading="lazy" decoding="async" />
                        <figcaption>"The useful conversation is usually one hello away."</figcaption>
                    </figure>
                </article>

                <section class="voices" aria-labelledby="voices-title">
                    <div class="voices__intro">
                        <p class="bend__number">"VOICES FROM UPSTREAM"</p>
                        <h2 id="voices-title">"The stories stay with you."</h2>
                    </div>
                    <div class="voices__quotes">
                        <blockquote>
                            <p>"It was just amazing to listen to AJ from Uber."</p>
                            <cite>"Anna Devcich / Soda"</cite>
                        </blockquote>
                        <blockquote>
                            <p>"Listening to the early days of Xero was so interesting!"</p>
                            <cite>"Dr Stuart Inglis / Altered Capital"</cite>
                        </blockquote>
                    </div>
                </section>

                <section class="whakatauki" aria-label="Waikato-Tainui whakatauki">
                    <p>"Waikato taniwharau"</p>
                    <blockquote>"He piko, he taniwha. He piko, he taniwha."</blockquote>
                    <span>"At every bend, strength, leadership and opportunity."</span>
                </section>

                <article class="bend bend--courage" id="next-bend">
                    <div class="bend__copy bank--west">
                        <p class="bend__number">"06 / THE NEXT BEND"</p>
                        <p class="kicker">"Step outside the familiar"</p>
                        <h2>"You do not need a pitch. You need a first sentence."</h2>
                        <p>"Come curious. Ask someone what they are building. Tell them what you are stuck on. Offer one connection. That is how a network becomes a community."</p>
                    </div>
                    <figure class="bend__media bank--east">
                        <img src="assets/woman-founder.webp" alt="A woman founder celebrating an early business milestone" loading="lazy" decoding="async" />
                        <figcaption>"We celebrate the sketch, the prototype and the first customer."</figcaption>
                    </figure>
                </article>

                <section class="stage-picker" aria-labelledby="stage-title">
                    <div>
                        <p class="bend__number">"07 / WHERE ARE YOU NOW?"</p>
                        <h2 id="stage-title">"There is a place for your stage."</h2>
                    </div>
                    <div class="stage-picker__controls" role="group" aria-label="Choose your business stage">
                        {stage_prompts.iter().enumerate().map(|(index, (label, _))| {
                            view! {
                                <button
                                    type="button"
                                    class:active=move || selected_stage.get() == index
                                    on:click=move |_| selected_stage.set(index)
                                >{*label}</button>
                            }
                        }).collect_view()}
                    </div>
                    <p class="stage-picker__answer">{move || stage_prompts[selected_stage.get()].1}</p>
                </section>

                <section class="investors" aria-labelledby="investors-title">
                    <div class="investors__body">
                        <p class="bend__number">"08 / INVESTORS"</p>
                        <p class="kicker">"Capital starts with a conversation"</p>
                        <h2 id="investors-title">"Are you an investor? Come and meet the people building next."</h2>
                        <p>"Meet founders and soon-to-be founders. Hear the idea in their own words, ask what they have learned and discuss what could make it stronger. Maybe you can help them."</p>
                        <a class="text-link" href=EVENTBRITE_URL target="_blank" rel="noreferrer">"Join the room "<span>"↗"</span></a>
                    </div>
                    <div class="investors__prompt">
                        <p>"Support can look like"</p>
                        <strong>"A useful question"</strong>
                        <strong>"A trusted introduction"</strong>
                        <strong>"A real partnership"</strong>
                        <strong>"Financial backing"</strong>
                    </div>
                </section>

                <section class="vision" id="kirikiriroa" aria-labelledby="vision-title">
                    <div class="vision__lead">
                        <p class="bend__number">"09 / KIRIKIRIROA"</p>
                        <p class="kicker">"The convening vision"</p>
                        <h2 id="vision-title">"Build the business community we wish we had met sooner."</h2>
                    </div>
                    <div class="vision__body">
                        <p class="vision__rally">"Across Waikato, entrepreneurs are building some of the country's best companies. They deserve moral support, partnership, practical help and financial backing."</p>
                        <p>"Dr Stuart Inglis brings Waikato roots, engineering and computer science connections, and the perspective of Altered Capital to help University of Waikato and Wintec collaborate."</p>
                        <p>"The invitation is deliberately wide: experienced founders, first-time founders, future founders, students and new graduates. We are here to be cheerleaders for the next brave step."</p>
                    </div>
                </section>

                <section class="hosts" id="hosts" aria-labelledby="hosts-title">
                    <div class="hosts__intro">
                        <p class="bend__number">"10 / YOUR HOSTS"</p>
                        <h2 id="hosts-title">"People who will make the introduction."</h2>
                    </div>
                    <div class="host-list">
                        <a href="https://alteredcapital.com" target="_blank" rel="noreferrer">
                            <span>"01"</span><strong>"Dr Stuart Inglis"</strong><small>"Altered Capital / organiser"</small><b>"↗"</b>
                        </a>
                        <a href="mailto:anna@sodainc.com">
                            <span>"02"</span><strong>"Anna Devcich"</strong><small>"Soda"</small><b>"↗"</b>
                        </a>
                        <a href="mailto:myles.mcinnes@waikato.ac.nz">
                            <span>"03"</span><strong>"Myles McInnes"</strong><small>"Hiko Hub"</small><b>"↗"</b>
                        </a>
                        <a href="mailto:john@sodainc.com">
                            <span>"04"</span><strong>"John O'Donoghue"</strong><small>"Soda"</small><b>"↗"</b>
                        </a>
                    </div>
                </section>

                <section class="partners" id="partners" aria-labelledby="partners-title">
                    <div>
                        <p class="bend__number">"11 / TOGETHER"</p>
                        <h2 id="partners-title">"Brought into the same current by"</h2>
                        <p class="partner-credit">
                            <a href="https://www.hikohub.co.nz" target="_blank" rel="noreferrer">"Hiko Hub (University of Waikato)"</a>
                            <i>"/"</i>
                            <a href="https://www.sodainc.com" target="_blank" rel="noreferrer">"Soda (Wintec)"</a>
                            <i>"/"</i>
                            <a href="https://alteredcapital.com" target="_blank" rel="noreferrer">"Altered Capital"</a>
                        </p>
                    </div>
                    <div class="partner-grid">
                        <div class="partner-group">
                            <p>"Hiko Hub "<span>"(University of Waikato)"</span></p>
                            <div>
                                <PartnerLogo href="https://www.hikohub.co.nz" src="assets/hiko-logo.png" alt="Hiko Hub" />
                                <PartnerLogo href="https://www.waikato.ac.nz" src="assets/waikato-logo.svg" alt="University of Waikato" />
                            </div>
                        </div>
                        <div class="partner-group">
                            <p>"Soda "<span>"(Wintec)"</span></p>
                            <div>
                                <PartnerLogo href="https://www.sodainc.com" src="assets/soda-logo.svg" alt="Soda" />
                                <PartnerLogo href="https://www.wintec.ac.nz" src="assets/wintec-logo.svg" alt="Wintec" />
                            </div>
                        </div>
                        <div class="partner-group">
                            <p>"Altered Capital"</p>
                            <div>
                                <PartnerLogo href="https://alteredcapital.com" src="assets/altered-one.png" alt="Altered Capital" />
                            </div>
                        </div>
                    </div>
                </section>
            </section>

            <footer class="sea" id="port" aria-labelledby="sea-title">
                <img src="assets/port-waikato.jpg" alt="The Waikato River meeting the Tasman Sea at Port Waikato" loading="lazy" decoding="async" />
                <div class="sea__veil"></div>
                <div class="sea__content">
                    <p class="bend__number">"TE PUUAAHA O WAIKATO / SEA LEVEL"</p>
                    <h2 id="sea-title">"Meet the people who move an idea into the world."</h2>
                    <a class="sea__cta" href=EVENTBRITE_URL target="_blank" rel="noreferrer">
                        <span>"Find Entrepreneur Meetup on Eventbrite"</span><b>"↗"</b>
                    </a>
                    <p class="sea__welcome">"Kia ora. Namaste. Ni hao. Talofa. Welcome."</p>
                </div>
                <div class="sea__fineprint">
                    <span>"Waikato Entrepreneur Meetup / Kirikiriroa, Aotearoa"</span>
                    <span>"River geometry: OpenStreetMap contributors, ODbL"</span>
                </div>
            </footer>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
