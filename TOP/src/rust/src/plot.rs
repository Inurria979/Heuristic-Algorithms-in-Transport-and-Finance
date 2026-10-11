//! PNG figures with plotters: the routes of one solution, the total reward of the alpha grid
//! and the heatmap of relative rewards. Colours follow the validated reference palette
//! (light mode): categorical hues in fixed order for routes, a one-hue blue ramp for the
//! heatmap.

use crate::alpha_grid::{grid_alpha, ALPHAS, GRID_TEXT};
use crate::model::{set_of, Instance, Solution};
use plotters::prelude::*;
use std::fs;
use std::path::Path;

/// Categorical colours of the routes, in fixed order; at most 4 routes in the benchmark, so
/// the order never wraps there.
const ROUTE_COLORS: [RGBColor; 8] = [
    RGBColor(0x2a, 0x78, 0xd6),
    RGBColor(0xeb, 0x68, 0x34),
    RGBColor(0x1b, 0xaf, 0x7a),
    RGBColor(0xed, 0xa1, 0x00),
    RGBColor(0xe8, 0x7b, 0xa4),
    RGBColor(0x00, 0x83, 0x00),
    RGBColor(0x4a, 0x3a, 0xa7),
    RGBColor(0xe3, 0x49, 0x48),
];
const SURFACE: RGBColor = RGBColor(0xfc, 0xfc, 0xfb);
const INK: RGBColor = RGBColor(0x0b, 0x0b, 0x0b);
const INK_2: RGBColor = RGBColor(0x52, 0x51, 0x4e);
const MUTED: RGBColor = RGBColor(0xb4, 0xb2, 0xab);
const GRID: RGBColor = RGBColor(0xe6, 0xe5, 0xe0);

/// Colour bins of the heatmap: reward of an alpha relative to the best reward of the
/// instance (sequential blue ramp, darkest = optimal).
const HEAT_BINS: [(f64, &str, RGBColor); 6] = [
    (1.0, "optimal (100 %)", RGBColor(0x0d, 0x36, 0x6b)),
    (0.98, ">= 98 %", RGBColor(0x1c, 0x5c, 0xab)),
    (0.95, ">= 95 %", RGBColor(0x2a, 0x78, 0xd6)),
    (0.90, ">= 90 %", RGBColor(0x55, 0x98, 0xe7)),
    (0.80, ">= 80 %", RGBColor(0x9e, 0xc5, 0xf4)),
    (0.0, "< 80 %", RGBColor(0xcd, 0xe2, 0xfb)),
];

/// Register a system TrueType font as "sans-serif" for plotters (it has no built-in font),
/// once per process, so that plots drawn in parallel share it.
fn register_font() -> Result<(), String> {
    static REGISTERED: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
    REGISTERED.get_or_init(load_font).clone()
}

/// Read the first TrueType font found and register it with plotters.
fn load_font() -> Result<(), String> {
    const CANDIDATES: [&str; 5] = [
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/segoeui.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
    ];
    let bytes = CANDIDATES
        .iter()
        .find_map(|p| fs::read(p).ok())
        .ok_or("no TrueType font found for the plot (looked for Arial, Segoe UI, DejaVu Sans)")?;
    plotters::style::register_font("sans-serif", FontStyle::Normal, Box::leak(bytes.into_boxed_slice()))
        .map_err(|_| "cannot load the plot font".to_string())
}

/// Draw the solution of one instance to a PNG: every route in its own colour from the
/// source (square) to the sink (diamond), visited customers in the colour of their route,
/// unvisited customers in grey and customers out of range as hollow grey circles. Marker
/// area grows with the reward; every customer carries its node number. `note` is the
/// subtitle (how the solution was obtained).
pub fn plot_solution(inst: &Instance, sol: &Solution, note: &str, out: &Path) -> Result<(), String> {
    let err = |e: &dyn std::fmt::Debug| format!("{}: {e:?}", out.display());
    register_font()?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let (n, sink) = (inst.n(), inst.sink());
    let (w, h, panel_w) = (1400u32, 1000u32, 330u32);
    let root = BitMapBackend::new(out, (w, h)).into_drawing_area();
    root.fill(&SURFACE).map_err(|e| err(&e))?;

    // Same scale on both axes: pad the data box, then widen the short side to the aspect
    // ratio of the plotting area (figure minus legend panel, margins, caption, axis labels).
    let (margin, ylab, xlab, caption_h) = (24u32, 48u32, 36u32, 70u32);
    let (pw, ph) = ((w - panel_w - 2 * margin - ylab) as f64, (h - 2 * margin - xlab - caption_h) as f64);
    let fold = |f: fn(f64, f64) -> f64, v: &[f64]| v.iter().copied().reduce(f).unwrap();
    let (mut x0, mut x1) = (fold(f64::min, &inst.x), fold(f64::max, &inst.x));
    let (mut y0, mut y1) = (fold(f64::min, &inst.y), fold(f64::max, &inst.y));
    let pad = 0.05 * (x1 - x0).max(y1 - y0).max(1.0);
    (x0, x1, y0, y1) = (x0 - pad, x1 + pad, y0 - pad, y1 + pad);
    let scale = ((x1 - x0) / pw).max((y1 - y0) / ph);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    (x0, x1, y0, y1) = (cx - scale * pw / 2.0, cx + scale * pw / 2.0, cy - scale * ph / 2.0, cy + scale * ph / 2.0);

    let title = format!(
        "{}  ·  m = {}, tmax = {}  ·  reward {} of {}  ·  {} of {} customers visited",
        inst.name,
        inst.m,
        inst.tmax,
        sol.reward,
        inst.total_reward(),
        sol.visited(),
        n - 2
    );
    let (head, body) = root.split_vertically(caption_h);
    let (plot_area, panel) = body.split_horizontally(w - panel_w);
    head.draw(&Text::new(title, (margin as i32, 18), ("sans-serif", 22).into_font().color(&INK)))
        .map_err(|e| err(&e))?;
    head.draw(&Text::new(note, (margin as i32, 46), ("sans-serif", 15).into_font().color(&INK_2)))
        .map_err(|e| err(&e))?;

    let mut chart = ChartBuilder::on(&plot_area)
        .margin(margin)
        .x_label_area_size(xlab)
        .y_label_area_size(ylab)
        .build_cartesian_2d(x0..x1, y0..y1)
        .map_err(|e| err(&e))?;
    chart
        .configure_mesh()
        .bold_line_style(GRID)
        .light_line_style(TRANSPARENT)
        .axis_style(MUTED)
        .label_style(("sans-serif", 13).into_font().color(&INK_2))
        .draw()
        .map_err(|e| err(&e))?;

    // Marker radius: area proportional to the reward, 3..9 px.
    let max_reward = fold(f64::max, &inst.reward).max(1.0);
    let radius_of = |u: f64| (3.0 + 6.0 * (u / max_reward).sqrt()).round() as i32;
    let radius = |i: usize| radius_of(inst.reward[i]);
    let mut route_of = vec![None; n];
    for (k, r) in sol.routes.iter().enumerate() {
        for &i in &r.nodes {
            route_of[i] = Some(k);
        }
    }

    // Customers that are not visited: grey, hollow if even a single-customer route is too long.
    for i in inst.customers().filter(|&i| route_of[i].is_none()) {
        let style = if inst.reachable(i) { MUTED.filled() } else { MUTED.stroke_width(1) };
        chart.draw_series(std::iter::once(Circle::new((inst.x[i], inst.y[i]), radius(i), style))).map_err(|e| err(&e))?;
    }

    // Routes: 2 px legs source -> customers -> sink, then their customers on top with a
    // 2 px surface ring so that overlapping markers stay separate.
    for (k, r) in sol.routes.iter().enumerate() {
        let color = ROUTE_COLORS[k % ROUTE_COLORS.len()];
        let path: Vec<(f64, f64)> = std::iter::once(0)
            .chain(r.nodes.iter().copied())
            .chain(std::iter::once(sink))
            .map(|i| (inst.x[i], inst.y[i]))
            .collect();
        // One element per leg: a single thick polyline gets mitre spikes at sharp turns.
        chart
            .draw_series(path.windows(2).map(|leg| PathElement::new(leg.to_vec(), color.stroke_width(2))))
            .map_err(|e| err(&e))?;
    }
    for (k, r) in sol.routes.iter().enumerate() {
        let color = ROUTE_COLORS[k % ROUTE_COLORS.len()];
        for &i in &r.nodes {
            let p = (inst.x[i], inst.y[i]);
            chart
                .draw_series([Circle::new(p, radius(i) + 2, SURFACE.filled()), Circle::new(p, radius(i), color.filled())])
                .map_err(|e| err(&e))?;
        }
    }

    // Depots on top: source as a square, sink as a diamond (they coincide in set p7).
    let square = |c: (i32, i32), s: i32| Rectangle::new([(c.0 - s, c.1 - s), (c.0 + s, c.1 + s)], INK.filled());
    let diamond = |c: (i32, i32), s: i32| {
        Polygon::new(vec![(c.0, c.1 - s), (c.0 + s, c.1), (c.0, c.1 + s), (c.0 - s, c.1)], INK_2.filled())
    };
    chart
        .draw_series(std::iter::once(EmptyElement::at((inst.x[0], inst.y[0])) + square((0, 0), 7)))
        .map_err(|e| err(&e))?;
    chart
        .draw_series(std::iter::once(EmptyElement::at((inst.x[sink], inst.y[sink])) + diamond((0, 0), 9)))
        .map_err(|e| err(&e))?;

    // Node numbers, small and in secondary ink, to match the printed routes.
    let label_font = ("sans-serif", 11).into_font().color(&INK_2);
    chart
        .draw_series(inst.customers().map(|i| {
            let off = radius(i) + 2;
            EmptyElement::at((inst.x[i], inst.y[i])) + Text::new(i.to_string(), (off, -off - 9), label_font.clone())
        }))
        .map_err(|e| err(&e))?;

    // Legend panel to the right of the plot, so that it never hides a customer.
    let text = |s: String, y: i32, size: i32, color: &RGBColor| {
        Text::new(s, (52, y - size / 2 - 1), ("sans-serif", size).into_font().color(color))
    };
    let mut y = margin as i32 + 12;
    panel.draw(&text("Routes".into(), y, 16, &INK)).map_err(|e| err(&e))?;
    for (k, r) in sol.routes.iter().enumerate() {
        y += 46;
        let color = ROUTE_COLORS[k % ROUTE_COLORS.len()];
        panel
            .draw(&PathElement::new(vec![(14, y - 8), (40, y - 8)], color.stroke_width(3)))
            .map_err(|e| err(&e))?;
        panel.draw(&text(format!("route {}: {} customers", k + 1, r.nodes.len()), y - 8, 14, &INK)).map_err(|e| err(&e))?;
        panel
            .draw(&text(format!("reward {}, time {:.2} of {}", r.reward, r.time, inst.tmax), y + 10, 13, &INK_2))
            .map_err(|e| err(&e))?;
    }
    if sol.routes.is_empty() {
        y += 30;
        panel.draw(&text("no feasible route".into(), y, 14, &INK_2)).map_err(|e| err(&e))?;
    }
    y += 56;
    panel.draw(&text("Nodes".into(), y, 16, &INK)).map_err(|e| err(&e))?;
    let r_mid = radius_of(max_reward / 2.0);
    let items = [
        "source (node 0)".to_string(),
        format!("sink (node {sink})"),
        "customer on a route (route colour)".to_string(),
        "customer not visited".to_string(),
        "customer out of range".to_string(),
    ];
    for (kind, label) in items.into_iter().enumerate() {
        y += 30;
        let c = (27, y - 1);
        match kind {
            0 => panel.draw(&square(c, 6)),
            1 => panel.draw(&diamond(c, 8)),
            2 => panel.draw(&Circle::new(c, r_mid, ROUTE_COLORS[0].filled())),
            3 => panel.draw(&Circle::new(c, r_mid, MUTED.filled())),
            _ => panel.draw(&Circle::new(c, r_mid, MUTED.stroke_width(1))),
        }
        .map_err(|e| err(&e))?;
        panel.draw(&text(label, y, 14, &INK)).map_err(|e| err(&e))?;
    }
    y += 34;
    let min_reward = fold(f64::min, &inst.reward[1..sink]);
    panel
        .draw(&text(format!("marker area proportional to reward ({min_reward}..{max_reward})"), y, 13, &INK_2))
        .map_err(|e| err(&e))?;
    panel
        .draw(&text(format!("{} of {} customers out of range", n - 2 - inst.reachable_count(), n - 2), y + 22, 13, &INK_2))
        .map_err(|e| err(&e))?;
    root.present().map_err(|e| err(&e))
}

/// Line chart of the total reward of all instances when every instance uses the same alpha
/// of the grid: one line per series (deterministic, mean of the GRASP runs of each
/// randomisation), in categorical order and direct-labelled at its right end, plus dashed
/// reference levels (e.g. the best alpha of every instance) labelled on the left.
pub fn plot_total_reward(series: &[(String, Vec<f64>)], references: &[(String, f64)], out: &Path) -> Result<(), String> {
    let err = |e: &dyn std::fmt::Debug| format!("{}: {e:?}", out.display());
    register_font()?;
    let root = BitMapBackend::new(out, (1300, 680)).into_drawing_area();
    root.fill(&SURFACE).map_err(|e| err(&e))?;
    let values = series.iter().flat_map(|s| s.1.iter().copied()).chain(references.iter().map(|r| r.1));
    let (lo, hi) = values.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(v), b.max(v)));
    let span = (hi - lo).max(1.0);
    let x_hi = grid_alpha(ALPHAS - 1) + 0.22;
    let mut chart = ChartBuilder::on(&root)
        .caption("Total reward of all instances with the same alpha for every instance", ("sans-serif", 22).into_font().color(&INK))
        .margin(24)
        .x_label_area_size(48)
        .y_label_area_size(80)
        .build_cartesian_2d(0.0..x_hi, (lo - 0.06 * span)..(hi + 0.08 * span))
        .map_err(|e| err(&e))?;
    chart
        .configure_mesh()
        .bold_line_style(GRID)
        .light_line_style(TRANSPARENT)
        .axis_style(MUTED)
        .x_desc("alpha")
        .y_desc("total reward")
        .x_labels(11)
        .x_label_formatter(&|v| if *v <= 1.0 + 1e-9 { format!("{v:.1}") } else { String::new() })
        .y_label_formatter(&|v| format!("{v:.0}"))
        .label_style(("sans-serif", 14).into_font().color(&INK_2))
        .axis_desc_style(("sans-serif", 15).into_font().color(&INK_2))
        .draw()
        .map_err(|e| err(&e))?;
    let label = ("sans-serif", 14).into_font().color(&INK_2);
    for (text, level) in references {
        chart
            .draw_series(DashedLineSeries::new(vec![(0.0, *level), (grid_alpha(ALPHAS - 1), *level)], 8, 5, INK_2.stroke_width(2)))
            .map_err(|e| err(&e))?;
        chart
            .draw_series(std::iter::once(Text::new(format!("{text}: {level}"), (0.01, *level + 0.025 * span), label.clone())))
            .map_err(|e| err(&e))?;
    }
    for (s, (name, totals)) in series.iter().enumerate() {
        let color = ROUTE_COLORS[s];
        let pts: Vec<(f64, f64)> = totals.iter().enumerate().map(|(k, &t)| (grid_alpha(k), t)).collect();
        chart.draw_series(LineSeries::new(pts.clone(), color.stroke_width(2))).map_err(|e| err(&e))?;
        chart
            .draw_series(pts.iter().map(|&p| Circle::new(p, 4, color.filled())))
            .map_err(|e| err(&e))?;
        let last = *pts.last().unwrap();
        chart
            .draw_series(std::iter::once(Text::new(name.clone(), (last.0 + 0.012, last.1), ("sans-serif", 14).into_font().color(&INK))))
            .map_err(|e| err(&e))?;
    }
    root.present().map_err(|e| err(&e))
}

/// Heatmap of reward(alpha) / best reward, one row per instance (`names`, grouped by set)
/// and one column per alpha of the grid.
pub fn plot_heatmap(names: &[&str], rel: &[Vec<f64>], out: &Path) -> Result<(), String> {
    let err = |e: &dyn std::fmt::Debug| format!("{}: {e:?}", out.display());
    register_font()?;
    let (row_h, gap, left, top) = (3i32, 14i32, 110i32, 90i32);
    let cell_w = 900 / ALPHAS as i32;
    let sets: Vec<&str> = names.iter().fold(Vec::new(), |mut acc, n| {
        if acc.last() != Some(&set_of(n)) {
            acc.push(set_of(n));
        }
        acc
    });
    let height = top + names.len() as i32 * row_h + sets.len() as i32 * gap + 60;
    let width = left + ALPHAS as i32 * cell_w + 220;
    let root = BitMapBackend::new(out, (width as u32, height as u32)).into_drawing_area();
    root.fill(&SURFACE).map_err(|e| err(&e))?;
    let font = |size: i32, c: &RGBColor| ("sans-serif", size).into_font().color(c);
    root.draw(&Text::new("Reward of every alpha relative to the best alpha of the instance", (24, 18), font(22, &INK)))
        .map_err(|e| err(&e))?;
    root.draw(&Text::new(
        format!("one row per instance with a feasible route, grouped by set; one column per alpha = {GRID_TEXT}"),
        (24, 48),
        font(15, &INK_2),
    ))
    .map_err(|e| err(&e))?;
    let bin = |r: f64| HEAT_BINS.iter().find(|b| r >= b.0 - 1e-12).unwrap().2;
    let mut y = top;
    for set in &sets {
        let rows: Vec<usize> = (0..names.len()).filter(|&i| set_of(names[i]) == *set).collect();
        root.draw(&Text::new(format!("{set} ({})", rows.len()), (24, y + rows.len() as i32 * row_h / 2 - 8), font(14, &INK)))
            .map_err(|e| err(&e))?;
        for &i in &rows {
            for (k, &r) in rel[i].iter().enumerate() {
                let x = left + k as i32 * cell_w;
                root.draw(&Rectangle::new([(x, y), (x + cell_w, y + row_h)], bin(r).filled())).map_err(|e| err(&e))?;
            }
            y += row_h;
        }
        y += gap;
    }
    for k in (0..ALPHAS).step_by(2) {
        let x = left + k as i32 * cell_w + cell_w / 2 - 12;
        root.draw(&Text::new(format!("{:.2}", grid_alpha(k)), (x, y), font(13, &INK_2))).map_err(|e| err(&e))?;
    }
    root.draw(&Text::new("alpha", (left + ALPHAS as i32 * cell_w / 2 - 15, y + 20), font(14, &INK_2)))
        .map_err(|e| err(&e))?;
    let lx = left + ALPHAS as i32 * cell_w + 30;
    root.draw(&Text::new("reward / best", (lx, top), font(14, &INK))).map_err(|e| err(&e))?;
    for (b, (_, label, color)) in HEAT_BINS.iter().enumerate() {
        let ly = top + 28 + b as i32 * 26;
        root.draw(&Rectangle::new([(lx, ly), (lx + 18, ly + 18)], color.filled())).map_err(|e| err(&e))?;
        root.draw(&Text::new(*label, (lx + 28, ly + 2), font(14, &INK))).map_err(|e| err(&e))?;
    }
    root.present().map_err(|e| err(&e))
}
