use eframe::egui;

pub fn highlight_markdown_line(line: &str, job: &mut egui::text::LayoutJob) {
    let def = egui::Color32::from_rgb(220, 220, 220);
    if line.starts_with('#') {
        job.append(line, 0.0, egui::TextFormat { color: egui::Color32::from_rgb(255, 80, 80), font_id: egui::FontId::monospace(14.0), ..Default::default() });
    } else {
        process_inline_markdown(line, job, def);
    }
}

pub fn highlight_line_with_search(line: &str, job: &mut egui::text::LayoutJob, find_q: &str, use_md: bool) {
    let line_lower = line.to_lowercase();
    let mut last_idx = 0;
    
    for (start_idx, _) in line_lower.match_indices(find_q) {
        // Parte antes do match
        if start_idx > last_idx {
            let chunk = &line[last_idx..start_idx];
            if use_md {
                highlight_markdown_line(chunk, job);
            } else {
                job.append(chunk, 0.0, egui::TextFormat {
                    color: egui::Color32::from_rgb(200, 200, 200),
                    font_id: egui::FontId::monospace(14.0),
                    ..Default::default()
                });
            }
        }
        
        // O match (Destaque)
        let end_idx = start_idx + find_q.len();
        let match_chunk = &line[start_idx..end_idx];
        job.append(match_chunk, 0.0, egui::TextFormat {
            color: egui::Color32::BLACK,
            background: egui::Color32::from_rgb(255, 255, 0), // Amarelo
            font_id: egui::FontId::monospace(14.0),
            ..Default::default()
        });
        
        last_idx = end_idx;
    }
    
    // Parte final após o último match
    if last_idx < line.len() {
        let final_chunk = &line[last_idx..];
        if use_md {
            highlight_markdown_line(final_chunk, job);
        } else {
            job.append(final_chunk, 0.0, egui::TextFormat {
                color: egui::Color32::from_rgb(200, 200, 200),
                font_id: egui::FontId::monospace(14.0),
                ..Default::default()
            });
        }
    }
}

fn process_inline_markdown(text: &str, job: &mut egui::text::LayoutJob, def: egui::Color32) {
    let mut chars = text.chars().peekable();
    let mut curr = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                if !curr.is_empty() { job.append(&curr, 0.0, egui::TextFormat { color: def, font_id: egui::FontId::monospace(14.0), ..Default::default() }); curr.clear(); }
                let mut code = String::new();
                let mut found_close = false;
                while let Some(&nc) = chars.peek() { 
                    chars.next(); 
                    if nc == '`' { found_close = true; break; } 
                    code.push(nc); 
                }
                if found_close {
                    job.append(&format!("`{}`", code), 0.0, egui::TextFormat { color: egui::Color32::from_rgb(255, 200, 100), background: egui::Color32::from_rgb(40, 40, 40), font_id: egui::FontId::monospace(14.0), ..Default::default() });
                } else {
                    job.append(&format!("`{}", code), 0.0, egui::TextFormat { color: def, font_id: egui::FontId::monospace(14.0), ..Default::default() });
                }
            }
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut bold = String::new();
                let mut found_close = false;
                while let Some(&nc) = chars.peek() { 
                    if nc == '*' { 
                        chars.next(); 
                        if chars.peek() == Some(&'*') { 
                            chars.next(); 
                            found_close = true; 
                            break; 
                        } 
                        bold.push('*'); 
                    } else { 
                        chars.next(); 
                        bold.push(nc); 
                    } 
                }
                
                if found_close && !bold.is_empty() {
                    if !curr.is_empty() { job.append(&curr, 0.0, egui::TextFormat { color: def, font_id: egui::FontId::monospace(14.0), ..Default::default() }); curr.clear(); }
                    job.append(&format!("**{}**", bold), 0.0, egui::TextFormat { color: egui::Color32::from_rgb(100, 150, 255), font_id: egui::FontId::monospace(14.0), ..Default::default() });
                } else {
                    curr.push('*');
                    curr.push('*');
                    curr.push_str(&bold);
                    if found_close { curr.push('*'); curr.push('*'); }
                }
            }
            _ => { curr.push(ch); }
        }
    }
    if !curr.is_empty() { job.append(&curr, 0.0, egui::TextFormat { color: def, font_id: egui::FontId::monospace(14.0), ..Default::default() }); }
}
