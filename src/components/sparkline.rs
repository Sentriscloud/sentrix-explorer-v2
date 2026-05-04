//! Sparkline — minimal inline SVG line chart.
//!
//! No D3, no chart lib. The geometry is 5-line: scale points to a
//! viewBox, emit `<polyline>`, drop a soft area-fill underneath, plot
//! a dot at the latest point. Costs ≤ 4 SVG nodes per sparkline.

use leptos::prelude::*;

#[component]
pub fn Sparkline(
    /// Y-values, oldest first → newest last. Empty input renders a
    /// flat baseline (no error state).
    points: Vec<f64>,
    /// Stroke + fill color. Pass any valid CSS color string.
    #[prop(default = "#DBC17F")]
    stroke: &'static str,
    /// Pixel width / height of the rendered SVG.
    #[prop(default = 96)]
    width: u32,
    #[prop(default = 24)] height: u32,
) -> impl IntoView {
    let n = points.len();
    let view_box = format!("0 0 {width} {height}");

    if n < 2 {
        // One or zero datapoints — paint a centre baseline so the
        // slot doesn't visually pop in once data accumulates.
        return view! {
            <svg width=width.to_string() height=height.to_string() viewBox=view_box.clone()>
                <line
                    x1="0" y1=(height as f64 / 2.0).to_string()
                    x2=width.to_string() y2=(height as f64 / 2.0).to_string()
                    stroke="#3f3f46"
                    stroke-width="1"
                />
            </svg>
        }
        .into_any();
    }

    let min = points.iter().copied().fold(f64::INFINITY, f64::min);
    let max = points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-9);

    let step = width as f64 / (n - 1) as f64;
    let pad_y = 2.0;
    let usable_h = height as f64 - 2.0 * pad_y;

    let mapped: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f64 * step;
            let y = pad_y + (1.0 - ((v - min) / span)) * usable_h;
            (x, y)
        })
        .collect();

    let polyline_pts = mapped
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");

    // Area fill polygon — same path plus the two bottom corners.
    let area_pts = format!(
        "0,{h} {pl} {w},{h}",
        h = height,
        pl = polyline_pts,
        w = width,
    );

    let last = mapped.last().copied().unwrap_or((0.0, 0.0));

    view! {
        <svg width=width.to_string() height=height.to_string() viewBox=view_box>
            <polygon points=area_pts fill=stroke fill-opacity="0.12" />
            <polyline
                points=polyline_pts
                fill="none"
                stroke=stroke
                stroke-width="1.5"
                stroke-linejoin="round"
                stroke-linecap="round"
            />
            <circle cx=last.0.to_string() cy=last.1.to_string() r="2" fill=stroke />
        </svg>
    }
    .into_any()
}
