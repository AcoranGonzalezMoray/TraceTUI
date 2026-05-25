use super::theme::THEME;
use crate::app::{App, AppConnection, InvestigationReport, SidebarFocus};
use crate::config;
use crate::tr;
use crate::utils::formatting;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Chart, LineGauge, Paragraph,
        Row, Sparkline, Table, TableState,
    },
};
pub fn render_center_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Center;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };
    if app.investigation.is_investigating {
        render_loading_screen(f, app, area, border_color);
        return;
    }
    if app.ui.is_initial_loading {
        render_initial_loading_screen(
            f,
            area,
            border_color,
            border_type,
            app.ui.frame_count,
            &app.ui.translator,
        );
        return;
    }
    if app.ui.show_map {
        if let Some(repo) = &app.investigation.investigation_report {
            render_map_view(f, app, repo, area, border_color);
            return;
        }
    }
    if let Some(repo) = &app.investigation.investigation_report {
        render_investigation_report(f, app, repo, area, border_color);
        return;
    }
    if area.height < 10 || area.width < 30 {
        return;
    }
    if let Some(selected_app) = app.get_selected_app() {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);
        render_process_info_section(f, app, selected_app, sections[0], border_color, border_type);
        render_center_tabs(f, app, sections[1], border_color);
        match app.ui.center_tab {
            1 => render_risk_barchart(f, app, sections[2], border_color, border_type),
            2 => render_timeline_chart(f, app, sections[2], border_color, border_type),
            _ => render_connections_section(
                f,
                app,
                selected_app,
                sections[2],
                border_color,
                border_type,
            ),
        }
    } else {
        render_no_selection_view(f, app, area, border_color, border_type);
    }
}
fn render_initial_loading_screen(
    f: &mut ratatui::Frame,
    area: Rect,
    _border_color: Color,
    _border_type: BorderType,
    frame_count: u64,
    t: &crate::i18n::Translator,
) {
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let s = spinner[(frame_count as usize) % spinner.len()];
    let paragraphs = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!(" {} {}", s, tr!(t, "center.initial_loading")),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("   {}", tr!(t, "center.initial_wait")),
            Style::default()
                .fg(THEME.text_dim)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    let p = Paragraph::new(paragraphs)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(p, area);
}
fn render_loading_screen(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    _border_color: ratatui::style::Color,
) {
    let t = &app.ui.translator;
    let loading_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 󰩠 {} ", tr!(t, "center.loading_title")))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.warning))
        .border_type(BorderType::Thick);

    let loading_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("   {} ", tr!(t, "center.loading_diag")),
            Style::default().fg(THEME.text_main),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("   > {}", tr!(t, "center.loading_nslookup")),
            Style::default().fg(THEME.text_dim),
        )]),
        Line::from(vec![Span::styled(
            format!("   > {}", tr!(t, "center.loading_ping")),
            Style::default().fg(THEME.text_dim),
        )]),
        Line::from(vec![Span::styled(
            format!("   > {}", tr!(t, "center.loading_tracert")),
            Style::default().fg(THEME.text_dim),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("   {} ", tr!(t, "center.loading_wait")),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];
    let p = Paragraph::new(loading_text).block(loading_block);
    f.render_widget(p, area);
}
fn render_investigation_report(
    f: &mut ratatui::Frame,
    app: &App,
    repo: &InvestigationReport,
    area: Rect,
    _border_color: ratatui::style::Color,
) {
    let risk_color = if repo.risk_score < 30 {
        THEME.success
    } else if repo.risk_score < 60 {
        THEME.warning
    } else {
        THEME.danger
    };
    let t = &app.ui.translator;
    let dashboard_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);
    render_circular_gauge(f, t, repo, dashboard_layout[0], risk_color);
    render_security_remarks(f, app, repo, dashboard_layout[1], risk_color);
    render_network_info(f, t, repo, dashboard_layout[2], risk_color);
    render_network_route_table(f, t, repo, dashboard_layout[3]);
}
fn render_circular_gauge(
    f: &mut ratatui::Frame,
    t: &crate::i18n::Translator,
    repo: &InvestigationReport,
    area: Rect,
    risk_color: Color,
) {
    use ratatui::widgets::canvas::{Canvas, Points};
    use std::f64::consts::PI;
    let ratio = repo.risk_score as f64 / 100.0;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "investigation.trust_score")))
        .title_style(Style::default().fg(risk_color).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(risk_color))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let vw = inner.width as f64 * 2.0;
    let vh = inner.height as f64 * 4.0;
    let half = 1.0;
    let (x_bounds, y_bounds) = if vw >= vh {
        ([-half * vw / vh, half * vw / vh], [-half, half])
    } else {
        ([-half, half], [-half * vh / vw, half * vh / vw])
    };
    let radius = 0.85;
    let thickness = 0.04;
    let canvas = Canvas::default()
        .x_bounds(x_bounds)
        .y_bounds(y_bounds)
        .marker(Marker::Braille)
        .paint(|ctx| {
            const N: usize = 80;
            let bg: Vec<(f64, f64)> = (0..N)
                .map(|i| {
                    let a = -PI / 2.0 + 2.0 * PI * i as f64 / N as f64;
                    (radius * a.cos(), radius * a.sin())
                })
                .collect();
            ctx.draw(&Points {
                coords: &bg,
                color: Color::DarkGray,
            });
            let active_n = (N as f64 * ratio).ceil() as usize;
            if active_n > 0 {
                for &r in &[
                    radius - thickness * 2.0,
                    radius - thickness,
                    radius,
                    radius + thickness,
                    radius + thickness * 2.0,
                ] {
                    let pts: Vec<(f64, f64)> = (0..=active_n)
                        .map(|i| {
                            let a = -PI / 2.0 + 2.0 * PI * i as f64 / N as f64;
                            (r * a.cos(), r * a.sin())
                        })
                        .collect();
                    ctx.draw(&Points {
                        coords: &pts,
                        color: risk_color,
                    });
                }
            }
        });
    f.render_widget(canvas, inner);
    let risk_label = if repo.risk_score < 30 {
        tr!(t, "investigation.risk_low")
    } else if repo.risk_score < 60 {
        tr!(t, "investigation.risk_medium")
    } else {
        tr!(t, "investigation.risk_high")
    };
    let text_lines: usize = 2;
    let pad = (inner.height.saturating_sub(text_lines as u16)) / 2;
    let mut overlay: Vec<Line> = Vec::with_capacity(pad as usize + text_lines);
    for _ in 0..pad as usize {
        overlay.push(Line::from(""));
    }
    overlay.push(Line::from(vec![Span::styled(
        format!("{}%", repo.risk_score),
        Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
    )]));
    overlay.push(Line::from(vec![Span::styled(
        risk_label,
        Style::default().fg(THEME.text_dim),
    )]));
    let overlay_p = Paragraph::new(overlay)
        .alignment(Alignment::Center)
        .style(Style::default());
    f.render_widget(overlay_p, inner);
}
fn render_security_remarks(
    f: &mut ratatui::Frame,
    app: &App,
    repo: &InvestigationReport,
    area: Rect,
    risk_color: Color,
) {
    let t = &app.ui.translator;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "investigation.remarks")))
        .title_style(Style::default().fg(risk_color).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(risk_color))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let widths = [
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(20),
    ];
    let domain_ok = repo.domain.is_some()
        && !repo
            .risk_factors
            .iter()
            .any(|f| f.contains("Domain/Process mismatch") || f.contains("Hidden Identity"));
    let domain_val = repo.domain.clone().unwrap_or(tr!(t, "center.na"));
    let process_name = app
        .get_selected_app()
        .map(|a| a.process_name.as_str())
        .unwrap_or("");
    let known_ok = repo.domain.as_ref().is_some_and(|d| {
        config::DOMAIN_ALLOWLIST
            .iter()
            .any(|a| d.to_lowercase().contains(a))
    }) || config::KNOWN_SAFE_PROCESSES
        .iter()
        .any(|p| process_name.contains(p))
        || repo.risk_score < 20;
    let known_val = if known_ok {
        repo.organization.clone().unwrap_or(tr!(t, "center.known"))
    } else {
        tr!(t, "center.unrecognized")
    };
    let proxy_risk = repo
        .risk_factors
        .iter()
        .any(|f| f == config::RISK_FACTOR_PROXY || f == config::RISK_FACTOR_HOSTING);
    let proxy_val = if proxy_risk {
        if repo.proxy == Some(true) {
            tr!(t, "investigation.connection_proxy")
        } else {
            tr!(t, "investigation.connection_datacenter")
        }
    } else {
        tr!(t, "investigation.connection_direct")
    };
    let low_latency = repo.ping_ms.as_ref().is_some_and(|ms| {
        ms.parse::<u32>()
            .ok()
            .is_some_and(|v| v < config::INV_RISK_LATENCY_THRESHOLD_MS)
    });
    let lat_val = repo
        .ping_ms
        .as_deref()
        .map(|v| format!("{}ms", v))
        .unwrap_or_else(|| tr!(t, "investigation.timeout"));
    let header_style = Style::default()
        .fg(THEME.text_dim)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from(""),
        Cell::from(tr!(t, "center.checkpoint")),
        Cell::from(tr!(t, "center.detail")),
    ])
    .style(header_style)
    .height(1);
    let rows = vec![
        Row::new(vec![
            Cell::from(if domain_ok { " ✓" } else { " ✗" }).style(Style::default().fg(
                if domain_ok {
                    THEME.success
                } else {
                    THEME.danger
                },
            )),
            Cell::from(tr!(t, "investigation.sec_domain")),
            Cell::from(domain_val).style(Style::default().fg(THEME.text_dim)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if known_ok { " ✓" } else { " ✗" }).style(Style::default().fg(
                if known_ok {
                    THEME.success
                } else {
                    THEME.danger
                },
            )),
            Cell::from(tr!(t, "investigation.sec_known")),
            Cell::from(known_val).style(Style::default().fg(THEME.text_dim)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if !proxy_risk { " ✓" } else { " ✗" }).style(Style::default().fg(
                if !proxy_risk {
                    THEME.success
                } else {
                    THEME.danger
                },
            )),
            Cell::from(tr!(t, "investigation.sec_proxy")),
            Cell::from(proxy_val).style(Style::default().fg(THEME.text_dim)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if low_latency { " ✓" } else { " ✗" }).style(Style::default().fg(
                if low_latency {
                    THEME.success
                } else {
                    THEME.danger
                },
            )),
            Cell::from(tr!(t, "investigation.sec_latency")),
            Cell::from(lat_val).style(Style::default().fg(THEME.text_dim)),
        ])
        .height(1),
    ];
    let table = Table::new(rows, widths).header(header);
    f.render_widget(table, inner);
}
fn render_network_info(
    f: &mut ratatui::Frame,
    t: &crate::i18n::Translator,
    repo: &InvestigationReport,
    area: Rect,
    risk_color: Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "investigation.network")))
        .title_style(Style::default().fg(risk_color).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(risk_color))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let widths = [
        Constraint::Length(3),
        Constraint::Length(14),
        Constraint::Fill(1),
    ];
    let header_style = Style::default()
        .fg(THEME.text_dim)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from(""),
        Cell::from(tr!(t, "center.field")),
        Cell::from(tr!(t, "center.value")),
    ])
    .style(header_style)
    .height(1);
    let yes = tr!(t, "investigation.yes").to_string();
    let no = tr!(t, "investigation.no").to_string();
    let rows = vec![
        Row::new(vec![
            Cell::from(""),
            Cell::from(tr!(t, "investigation.isp")),
            Cell::from(repo.isp.clone().unwrap_or(tr!(t, "center.na")))
                .style(Style::default().fg(THEME.text_main)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(""),
            Cell::from(tr!(t, "investigation.org")),
            Cell::from(repo.organization.clone().unwrap_or(tr!(t, "center.na")))
                .style(Style::default().fg(THEME.text_main)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(""),
            Cell::from(tr!(t, "investigation.as")),
            Cell::from(repo.as_info.clone().unwrap_or(tr!(t, "center.na")))
                .style(Style::default().fg(THEME.text_main)),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if repo.proxy.unwrap_or(false) {
                " ⚠"
            } else {
                ""
            })
            .style(Style::default().fg(if repo.proxy.unwrap_or(false) {
                THEME.warning
            } else {
                THEME.success
            })),
            Cell::from(tr!(t, "center.proxy_vpn")),
            Cell::from(if repo.proxy.unwrap_or(false) {
                yes.as_str()
            } else {
                no.as_str()
            })
            .style(Style::default().fg(if repo.proxy.unwrap_or(false) {
                THEME.warning
            } else {
                THEME.success
            })),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if repo.hosting.unwrap_or(false) {
                " ⚠"
            } else {
                ""
            })
            .style(Style::default().fg(if repo.hosting.unwrap_or(false) {
                THEME.warning
            } else {
                THEME.success
            })),
            Cell::from(tr!(t, "center.hosting_dc")),
            Cell::from(if repo.hosting.unwrap_or(false) {
                yes.as_str()
            } else {
                no.as_str()
            })
            .style(Style::default().fg(if repo.hosting.unwrap_or(false) {
                THEME.warning
            } else {
                THEME.success
            })),
        ])
        .height(1),
        Row::new(vec![
            Cell::from(if repo.mobile.unwrap_or(false) {
                " ℹ"
            } else {
                ""
            })
            .style(Style::default().fg(if repo.mobile.unwrap_or(false) {
                THEME.accent
            } else {
                THEME.success
            })),
            Cell::from(tr!(t, "center.mobile")),
            Cell::from(if repo.mobile.unwrap_or(false) {
                yes.as_str()
            } else {
                no.as_str()
            })
            .style(Style::default().fg(if repo.mobile.unwrap_or(false) {
                THEME.accent
            } else {
                THEME.success
            })),
        ])
        .height(1),
    ];
    let table = Table::new(rows, widths).header(header);
    f.render_widget(table, inner);
}

fn render_network_route_table(
    f: &mut ratatui::Frame,
    t: &crate::i18n::Translator,
    repo: &InvestigationReport,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "investigation.route")))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let widths = [Constraint::Length(4), Constraint::Fill(1)];
    let header_style = Style::default()
        .fg(THEME.text_dim)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec![Cell::from("  #"), Cell::from(tr!(t, "center.hop"))])
        .style(header_style)
        .height(1);
    let rows: Vec<Row> = repo
        .hops
        .iter()
        .enumerate()
        .map(|(i, hop)| {
            Row::new(vec![
                Cell::from(format!(" {:>2}", i + 1)).style(Style::default().fg(THEME.secondary)),
                Cell::from(hop.clone()).style(Style::default().fg(THEME.text_main)),
            ])
            .height(1)
        })
        .collect();
    let table = Table::new(rows, widths).header(header);
    f.render_widget(table, inner);
}
fn render_map_view(
    f: &mut ratatui::Frame,
    app: &App,
    repo: &InvestigationReport,
    area: Rect,
    border_color: Color,
) {
    use ratatui::symbols::Marker;
    use ratatui::widgets::canvas::{Canvas, Map, MapResolution, Points};
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} {}:{} ",
            tr!(app.ui.translator, "map.title"),
            repo.ip,
            repo.port
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(BorderType::Thick);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let canvas = Canvas::default()
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .marker(Marker::Braille)
        .paint(|ctx| {
            ctx.draw(&Map {
                resolution: MapResolution::High,
                color: Color::White,
            });
            ctx.layer();

            let mut route: Vec<(f64, f64)> = Vec::new();
            if let Some(user) = &app.geo.user_geo {
                if user.lat != 0.0 || user.lon != 0.0 {
                    route.push((user.lon, user.lat));
                }
            }
            for (lon, lat) in &repo.hop_coords {
                route.push((*lon, *lat));
            }
            if repo.lat != 0.0 || repo.lon != 0.0 {
                route.push((repo.lon, repo.lat));
            }

            let segments = route.len().saturating_sub(1);
            if segments > 0 {
                let frames_per_seg = 40u64;
                let cycle = segments as u64 * frames_per_seg;
                let cf = app.ui.frame_count % cycle;
                let prog = cf as f64 / cycle as f64;
                let total = prog * segments as f64;
                let seg_idx = (total as usize).min(segments - 1);
                let seg_prog = total - seg_idx as f64;

                for i in 0..segments {
                    let (x1, y1) = route[i];
                    let (x2, y2) = route[i + 1];
                    ctx.draw(&ratatui::widgets::canvas::Line::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        Color::DarkGray,
                    ));
                }

                for i in 0..seg_idx {
                    let (x1, y1) = route[i];
                    let (x2, y2) = route[i + 1];
                    ctx.draw(&ratatui::widgets::canvas::Line::new(
                        x1,
                        y1,
                        x2,
                        y2,
                        Color::Cyan,
                    ));
                }

                let (x1, y1) = route[seg_idx];
                let (x2, y2) = route[seg_idx + 1];
                let cx = x1 + (x2 - x1) * seg_prog;
                let cy = y1 + (y2 - y1) * seg_prog;
                ctx.draw(&ratatui::widgets::canvas::Line::new(
                    x1,
                    y1,
                    cx,
                    cy,
                    Color::Cyan,
                ));

                ctx.draw(&Points {
                    coords: &[(cx, cy)],
                    color: Color::LightCyan,
                });
            }

            if let Some(user) = &app.geo.user_geo {
                if user.lat != 0.0 || user.lon != 0.0 {
                    ctx.draw(&Points {
                        coords: &[(user.lon, user.lat)],
                        color: Color::Green,
                    });
                }
            }

            if !repo.hop_coords.is_empty() {
                let hops: Vec<(f64, f64)> = repo
                    .hop_coords
                    .iter()
                    .map(|(lon, lat)| (*lon, *lat))
                    .collect();
                ctx.draw(&Points {
                    coords: &hops,
                    color: Color::Yellow,
                });
            }

            if repo.lat != 0.0 || repo.lon != 0.0 {
                ctx.draw(&Points {
                    coords: &[(repo.lon, repo.lat)],
                    color: Color::Red,
                });
            }
        });

    f.render_widget(canvas, inner);
}
fn render_process_info_section(
    f: &mut ratatui::Frame,
    app: &App,
    selected_app: &AppConnection,
    area: Rect,
    border_color: ratatui::style::Color,
    border_type: BorderType,
) {
    let t = &app.ui.translator;
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(3)])
        .split(area);
    let info_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 󰰍 {} ", tr!(t, "center.process_info")))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .margin(1)
        .split(main_layout[0]);
    let sig_info = match &selected_app.signature_status {
        crate::utils::signatures::SignatureStatus::Valid => {
            (format!("󰄬 {}", tr!(t, "center.signed")), THEME.success)
        }
        crate::utils::signatures::SignatureStatus::Invalid => {
            (format!("󰀪 {}", tr!(t, "center.corrupt")), THEME.danger)
        }
        crate::utils::signatures::SignatureStatus::Unsigned => {
            (format!("󰈸 {}", tr!(t, "center.unsigned")), THEME.warning)
        }
        crate::utils::signatures::SignatureStatus::Unknown => (
            format!("󰰍 {}", tr!(t, "center.unknown_sig")),
            THEME.text_dim,
        ),
    };
    let info_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  󰙅  {} ", tr!(t, "center.process_label")),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &selected_app.process_name,
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  󱔗  {} ", tr!(t, "center.pid_label")),
                Style::default().fg(THEME.primary),
            ),
            Span::styled(
                selected_app.pid.to_string(),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("    {} ", tr!(t, "center.risk_label")),
                Style::default().fg(THEME.primary),
            ),
            Span::styled(
                format!(" {} ", selected_app.risk_level),
                Style::default()
                    .fg(THEME.background)
                    .bg(
                        if selected_app.risk_level.contains("HIGH")
                            || selected_app.risk_level.contains("CRITICAL")
                        {
                            THEME.danger
                        } else if selected_app.risk_level.contains("MEDIUM") {
                            THEME.warning
                        } else {
                            THEME.success
                        },
                    )
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  󰄬  {}  ", tr!(t, "center.sig_label")),
                Style::default().fg(THEME.primary),
            ),
            Span::styled(
                &sig_info.0,
                Style::default().fg(sig_info.1).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let info = Paragraph::new(info_lines);
    f.render_widget(info_block, main_layout[0]);
    f.render_widget(info, top_chunks[0]);
    let gauge_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(top_chunks[1]);
    let cpu_ratio = (selected_app.cpu_usage as f64 / 100.0).clamp(0.0, 1.0);
    let cpu_color = if selected_app.cpu_usage > 50.0 {
        THEME.danger
    } else {
        THEME.success
    };
    let cpu_gauge = LineGauge::default()
        .block(
            Block::default()
                .title(format!(
                    " {} ",
                    tr!(t, "center.cpu", format!("{:.1}", selected_app.cpu_usage))
                ))
                .title_style(Style::default().fg(THEME.text_dim)),
        )
        .ratio(cpu_ratio);
    let cpu_gauge = if cpu_ratio > 0.0 {
        cpu_gauge.filled_style(Style::default().fg(cpu_color))
    } else {
        cpu_gauge
    };
    f.render_widget(cpu_gauge, gauge_layout[0]);
    let mem_ratio = (selected_app.memory_usage as f64 / config::MAX_MEMORY_BYTES).clamp(0.0, 1.0);
    let mem_gauge = LineGauge::default()
        .block(
            Block::default()
                .title(format!(
                    " {} ",
                    tr!(
                        t,
                        "center.mem",
                        formatting::format_bytes(selected_app.memory_usage)
                    )
                ))
                .title_style(Style::default().fg(THEME.text_dim)),
        )
        .ratio(mem_ratio);
    let mem_gauge = if mem_ratio > 0.0 {
        mem_gauge.filled_style(Style::default().fg(THEME.primary))
    } else {
        mem_gauge
    };
    f.render_widget(mem_gauge, gauge_layout[1]);
    if !app.trend.cpu_history.is_empty() {
        let spark_vals: Vec<u64> = app.trend.cpu_history.iter().map(|&v| v as u64).collect();
        let spark = Sparkline::default()
            .block(Block::default().title(format!(" {} ", tr!(t, "center.cpu_history"))))
            .style(Style::default().fg(cpu_color))
            .data(&spark_vals)
            .max(100);
        f.render_widget(spark, gauge_layout[2]);
    }
    let path_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    let path_str = &selected_app.process_path;
    let max_width = (main_layout[1].width as usize).saturating_sub(10);
    let display_path = if path_str.len() > max_width {
        format!("...{}", &path_str[path_str.len() - max_width..])
    } else {
        path_str.to_string()
    };
    let path_para = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  󰉋  {} ", tr!(t, "center.path_label")),
            Style::default().fg(THEME.primary),
        ),
        Span::styled(display_path, Style::default().fg(THEME.text_dim)),
    ]))
    .block(path_block);
    f.render_widget(path_para, main_layout[1]);
}
fn render_center_tabs(f: &mut ratatui::Frame, app: &App, area: Rect, border_color: Color) {
    let t = &app.ui.translator;
    let titles = [
        (1, tr!(t, "center.tab_connections")),
        (2, tr!(t, "center.tab_risk")),
        (3, tr!(t, "center.tab_timeline")),
    ];
    let mut spans = vec![Span::raw(" ")];
    for (i, (key, title)) in titles.iter().enumerate() {
        let active = i == app.ui.center_tab;
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            if active {
                format!("▎[{}] {} ", key, title)
            } else {
                format!(" [{}] {} ", key, title)
            },
            if active {
                Style::default()
                    .fg(THEME.background)
                    .bg(border_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(THEME.background)),
        area,
    );
}
fn render_risk_barchart(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    border_color: Color,
    border_type: BorderType,
) {
    let t = &app.ui.translator;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "center.risk_overview")))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let bar_data: Vec<(&str, u64)> = app
        .get_filtered_apps()
        .iter()
        .take(20)
        .map(|a| {
            let risk = match a.risk_level.as_str() {
                "CRITICAL" => 100u64,
                "HIGH" => 75,
                "MEDIUM" => 50,
                "LOW" => 25,
                _ => 10,
            };
            (a.process_name.as_str(), risk)
        })
        .collect();

    if bar_data.is_empty() {
        let p = Paragraph::new(tr!(t, "sidebar.select_hint"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim));
        f.render_widget(p, inner);
        return;
    }

    let bars: Vec<Bar> = app
        .get_filtered_apps()
        .iter()
        .take(20)
        .map(|a| {
            let risk = match a.risk_level.as_str() {
                "CRITICAL" => 100u64,
                "HIGH" => 75,
                "MEDIUM" => 50,
                "LOW" => 25,
                _ => 10,
            };
            let color = match a.risk_level.as_str() {
                "CRITICAL" | "HIGH" => THEME.danger,
                "MEDIUM" => THEME.warning,
                _ => THEME.success,
            };
            Bar::default()
                .value(risk)
                .label(a.process_name.as_str().into())
                .style(Style::default().fg(color))
        })
        .collect();

    let bar_count = bars.len().max(1) as u16;
    let bar_width =
        ((inner.width.saturating_sub(bar_count.saturating_sub(1))) / bar_count).clamp(3, 20);
    let bar_chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_gap(1)
        .bar_width(bar_width)
        .max(100);
    f.render_widget(bar_chart, inner);
}
fn render_timeline_chart(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    border_color: Color,
    border_type: BorderType,
) {
    let t = &app.ui.translator;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tr!(t, "center.timeline")))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.trend.conn_count_history.len() < 2 {
        let p = Paragraph::new(tr!(t, "center.timeline_wait"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim));
        f.render_widget(p, inner);
        return;
    }

    let max = *app.trend.conn_count_history.iter().max().unwrap_or(&1).max(&1);
    let data: Vec<(f64, f64)> = app
        .trend
        .conn_count_history
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v as f64))
        .collect();

    let dataset = ratatui::widgets::Dataset::default()
        .marker(Marker::Braille)
        .style(Style::default().fg(THEME.primary))
        .graph_type(ratatui::widgets::GraphType::Line)
        .data(&data);

    let chart = Chart::new(vec![dataset])
        .block(Block::default())
        .x_axis(
            ratatui::widgets::Axis::default()
                .bounds([0.0, app.trend.conn_count_history.len().saturating_sub(1) as f64])
                .labels(vec![
                    "0".into(),
                    format!("{}", app.trend.conn_count_history.len()),
                ]),
        )
        .y_axis(
            ratatui::widgets::Axis::default()
                .bounds([0.0, max as f64])
                .labels(vec!["0".into(), format!("{}", max)]),
        );

    f.render_widget(chart, inner);
}
fn render_connections_section(
    f: &mut ratatui::Frame,
    app: &App,
    selected_app: &AppConnection,
    area: Rect,
    border_color: ratatui::style::Color,
    border_type: BorderType,
) {
    let t = &app.ui.translator;
    if selected_app.connections.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                tr!(t, "center.no_conns"),
                Style::default().fg(THEME.text_dim),
            )]),
        ];
        let empty = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 󱂇 {} ", tr!(t, "center.connections", "")))
                    .title_style(
                        Style::default()
                            .fg(border_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color))
                    .border_type(border_type),
            )
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Center;
    let sel_bg = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let sel_fg = THEME.background;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 󱂇 {} ",
            tr!(t, "center.connections", selected_app.connections.len())
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    f.render_widget(block.clone(), area);
    let inner_area = block.inner(area);

    let widths = [
        Constraint::Length(8),
        Constraint::Length(5),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Fill(2),
    ];

    let header = Row::new(vec![
        Cell::from(""),
        Cell::from(tr!(t, "center.conn_proto")),
        Cell::from(tr!(t, "center.conn_local")),
        Cell::from(tr!(t, "center.conn_foreign")),
        Cell::from(tr!(t, "center.conn_state")),
        Cell::from(tr!(t, "center.conn_location")),
    ])
    .style(Style::default().fg(THEME.text_dim))
    .height(1);

    let rows: Vec<Row> = selected_app
        .connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let is_selected = i == app.network.selected_connection_index;
            let row_style = if is_selected {
                Style::default().bg(sel_bg)
            } else {
                Style::default()
            };

            let protocol_color = match conn.protocol.as_str() {
                "TCP" => THEME.success,
                "UDP" => THEME.warning,
                _ => THEME.secondary,
            };

            let indicator = Cell::from({
                if is_selected {
                    Span::styled(" ENTER ↵", Style::default().fg(sel_fg))
                } else {
                    Span::raw("        ")
                }
            });

            let proto = Cell::from(format!(" {} ", conn.protocol)).style(
                Style::default()
                    .fg(if is_selected {
                        sel_fg
                    } else {
                        THEME.background
                    })
                    .bg(protocol_color)
                    .add_modifier(Modifier::BOLD),
            );

            let local_fg = if is_selected { sel_fg } else { THEME.primary };
            let local = Cell::from(format!("{}:{}", conn.local_address, conn.local_port))
                .style(Style::default().fg(local_fg).add_modifier(Modifier::BOLD));

            let foreign_fg = if is_selected { sel_fg } else { THEME.accent };
            let foreign = Cell::from(format!("{}:{}", conn.foreign_address, conn.foreign_port))
                .style(Style::default().fg(foreign_fg));

            let state_txt = if is_selected {
                format!("{}  ↵", conn.state)
            } else {
                conn.state.clone()
            };
            let state = Cell::from(state_txt).style(Style::default().fg(if is_selected {
                sel_fg
            } else {
                THEME.text_dim
            }));

            let location = Cell::from(conn.location.as_deref().unwrap_or(""))
                .style(Style::default().fg(if is_selected { sel_fg } else { THEME.text_dim }));

            Row::new(vec![indicator, proto, local, foreign, state, location])
                .style(row_style)
                .height(1)
        })
        .collect();

    let mut table_state = TableState::default();
    table_state.select(Some(app.network.selected_connection_index));

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(Style::default())
        .highlight_symbol("");
    f.render_stateful_widget(table, inner_area, &mut table_state);
}
fn render_no_selection_view(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    border_color: ratatui::style::Color,
    border_type: BorderType,
) {
    let t = &app.ui.translator;
    let empty_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  ",
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tr!(t, "sidebar.select_hint"),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("", Style::default().fg(THEME.text_dim))]),
    ];
    let empty = Paragraph::new(empty_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 󰰍 {} ", tr!(t, "center.details")))
                .title_style(
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(border_color))
                .border_type(border_type),
        )
        .alignment(Alignment::Center);
    f.render_widget(empty, area);
}
