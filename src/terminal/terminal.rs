use macroquad::prelude::*;

// Логический размер терминала — должен совпадать с offscreen target
const TERM_W: f32 = 1280.0;
const TERM_H: f32 = 720.0;

const GREEN: Color =
    Color::new(
        0.18,
        1.0,
        0.28,
        0.94,
    );

pub struct InputResult {
    pub command:
        Option<String>,

    pub key_pressed:
        bool,

    pub printed_char:
        bool,
}

struct QueuedLine {
    text: String,
    visible: usize,
    accumulator: f32,
    speed: f32,
}

pub struct Terminal {
    pub lines:
        Vec<String>,

    pub input:
        String,

    history:
        Vec<String>,

    history_position:
        Option<usize>,

    queue:
        Vec<QueuedLine>,

    cursor_timer:
        f32,

    scroll:
        f32,

    rng:
        u32,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            lines:
                Vec::new(),

            input:
                String::new(),

            history:
                Vec::new(),

            history_position:
                None,

            queue:
                Vec::new(),

            cursor_timer:
                0.0,

            scroll:
                0.0,

            rng:
                0xA17C2026,
        }
    }

    pub fn print(
        &mut self,
        lines: &[&str],
    ) {
        for line in lines {
            self.lines.push(
                (*line).to_string(),
            );
        }
    }

    pub fn clear(
        &mut self,
    ) {
        self.lines.clear();
        self.queue.clear();
        self.scroll = 0.0;
    }

    pub fn is_busy(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn queue_slow(
        &mut self,
        lines: &[String],
    ) {
        for line in lines {
            self.queue.push(QueuedLine {
                text: line.clone(),
                visible: 0,
                accumulator: 0.0,
                speed: if line.is_empty() { 0.0 } else { 0.035 },
            });
        }
    }

    pub fn queue(
        &mut self,
        lines: &[String],
    ) {
        for line in lines {
            self.queue.push(
                QueuedLine {
                    text:
                        line.clone(),

                    visible: 0,

                    accumulator:
                        0.0,

                    speed:
                    if line.is_empty() {
                    0.0
                    } else {
                    0.012
                    },
                },
            );
        }
    }

    pub fn update_output(
        &mut self,
    ) -> bool {
        let mut character_printed = false;
        if let Some(line) =
            self.queue.first_mut()
        {
            let count =
                line.text
                    .chars()
                    .count();

            if line.visible < count {
                line.accumulator +=
                    get_frame_time();

                let step =
                    (
                        line.accumulator
                            / line.speed
                    )
                    .floor()
                    as usize;

                if step > 0 {
                    line.accumulator -=
                        step as f32
                            * line.speed;

                    line.visible =
                        (
                            line.visible
                                + step
                        )
                        .min(count);
                    character_printed = true;
                }
            } else {
                let done =
                    self.queue
                        .remove(0);

                self.lines.push(
                    done.text,
                );
            }
        }
        character_printed
    }

    pub fn update_input(
        &mut self,
    ) -> InputResult {
        self.cursor_timer +=
            get_frame_time();

        let printed_char = self.update_output();

        let mut key_pressed =
            false;

        let wheel =
            mouse_wheel().1;

        if wheel.abs() > 0.01 {
            self.scroll =
                (
                    self.scroll
                        + wheel * 3.0
                )
                .max(0.0);
        }

        while let Some(c) =
            get_char_pressed()
        {
            if !c.is_control() {
                self.input.push(c);
                key_pressed = true;
            }
        }

        if is_key_pressed(
            KeyCode::Backspace
        ) {
            self.input.pop();
            key_pressed = true;
        }

        if is_key_pressed(
            KeyCode::Enter
        ) {
            let command =
                self.input
                    .trim()
                    .to_string();

            self.lines.push(
                format!(
                    "> {}",
                    self.input
                ),
            );

            if !command.is_empty() {
                self.history.push(
                    command.clone(),
                );
            }

            self.input.clear();

            self.history_position =
                None;

            return InputResult {
                command:
                    Some(command),

                key_pressed:
                    true,
                printed_char:
                    false,
            };
        }

        if is_key_pressed(
            KeyCode::Up
        )
            && !self.history.is_empty()
        {
            let index =
                match self.history_position {
                    None =>
                        self.history.len()
                            - 1,

                    Some(0) => 0,

                    Some(value) =>
                        value - 1,
                };

            self.history_position =
                Some(index);

            self.input =
                self.history[index]
                    .clone();
        }

        if is_key_pressed(
            KeyCode::Down
        ) {
            if let Some(index) =
                self.history_position
            {
                if index + 1
                    < self.history.len()
                {
                    let next =
                        index + 1;

                    self.history_position =
                        Some(next);

                    self.input =
                        self.history[next]
                            .clone();
                } else {
                    self.history_position =
                        None;

                    self.input.clear();
                }
            }
        }

        if is_key_pressed(
            KeyCode::PageUp
        ) {
            self.scroll += 8.0;
        }

        if is_key_pressed(
            KeyCode::PageDown
        ) {
            self.scroll =
                (
                    self.scroll - 8.0
                )
                .max(0.0);
        }

        InputResult {
            command: None,
            key_pressed,
            printed_char,
        }
    }

    fn noise(
        &mut self,
    ) -> f32 {
        self.rng ^=
            self.rng << 13;

        self.rng ^=
            self.rng >> 17;

        self.rng ^=
            self.rng << 5;

        self.rng as f32
            / u32::MAX as f32
    }

    pub fn draw_terminal(
        &mut self,
        elapsed: f32,
        attention: bool,
        critical: bool,
    ) {
        // Используем фиксированный размер терминала, а не screen_width/height
        let width = TERM_W;
        let height = TERM_H;

        clear_background(
            Color::new(
                0.003,
                0.008,
                0.004,
                1.0,
            ),
        );

        let palette = if critical { Color::new(1.0, 0.16, 0.10, 0.94) } else { GREEN };

        let brightness =
            if self.noise() > 0.993 {
                0.84
            } else if self.noise() > 0.94 {
                0.96
            } else {
                1.0
            };

        let font_size =
            20.0;

        let line_height =
            font_size * 1.35;

        let top = 58.0;

        let bottom =
            height - 62.0;

        let max_lines =
            (
                (bottom - top)
                    / line_height
            )
            .floor()
            as usize;

        let total =
            self.lines.len()
                + usize::from(
                    !self.queue.is_empty()
                );

        let start =
            total.saturating_sub(
                max_lines
                    + self.scroll
                        as usize,
            );

        let end =
            (
                start
                    + max_lines
            )
            .min(total);

        for index in start..end {
            let y =
                top
                    + (
                        index - start
                    ) as f32
                        * line_height;

            let text =
                if index
                    < self.lines.len()
                {
                    self.lines[index]
                        .clone()
                } else {
                    self.queue
                        .first()
                        .map(|q| {
                            q.text
                                .chars()
                                .take(
                                    q.visible
                                )
                                .collect()
                        })
                        .unwrap_or_default()
                };

            draw_text_ex(
                &text,
                34.5,
                y + 0.5,
                TextParams {
                    font_size:
                        font_size as u16,

                    color:
                        Color::new(
                            palette.r,
                            palette.g,
                            palette.b,
                            0.10,
                        ),

                    ..Default::default()
                },
            );

            draw_text_ex(
                &text,
                34.0,
                y,
                TextParams {
                    font_size:
                        font_size as u16,

                    color:
                        Color::new(
                            palette.r,
                            palette.g,
                            palette.b,
                            0.92
                                * brightness,
                        ),

                    ..Default::default()
                },
            );
        }

        let prompt =
            format!(
                "> {}",
                self.input
            );

        draw_text_ex(
            &prompt,
            34.0,
            height - 28.0,
            TextParams {
                font_size:
                    font_size as u16,

                color:
                    Color::new(
                        palette.r,
                        palette.g,
                        palette.b,
                        brightness,
                    ),

                ..Default::default()
            },
        );

        if self.cursor_timer
            % 1.0
            < 0.55
        {
            let prompt_width =
                measure_text(
                    &prompt,
                    None,
                    font_size as u16,
                    1.0,
                )
                .width;

            draw_rectangle(
                36.0 + prompt_width,
                height
                    - font_size
                    - 25.0,
                11.0,
                font_size + 2.0,
                Color::new(
                    palette.r,
                    palette.g,
                    palette.b,
                    0.8
                        * brightness,
                ),
            );
        }

        draw_text_ex(
            "AURORA-17 // REMOTE OPERATIONS TERMINAL",
            34.0,
            40.0,
            TextParams {
                font_size: 15,
                color:
                    Color::new(
                        palette.r,
                        palette.g,
                        palette.b,
                        0.75
                            * brightness,
                    ),
                ..Default::default()
            },
        );

        if attention {
            let x =
                width - 300.0;

            let y = 34.0;

            draw_rectangle(
                x,
                y,
                266.0,
                54.0,
                Color::new(
                    0.01,
                    0.04,
                    0.015,
                    0.88,
                ),
            );

            draw_rectangle_lines(
                x,
                y,
                266.0,
                54.0,
                1.0,
                Color::new(
                    palette.r,
                    palette.g,
                    palette.b,
                    0.72,
                ),
            );

            draw_text_ex(
                "[ ! ] DIAGNOSTIC ATTENTION",
                x + 12.0,
                y + 22.0,
                TextParams {
                    font_size: 15,
                    color: palette,
                    ..Default::default()
                },
            );

            draw_text_ex(
                "RUN DIAGNOSE",
                x + 12.0,
                y + 42.0,
                TextParams {
                    font_size: 12,
                    color:
                        Color::new(
                            palette.r,
                            palette.g,
                            palette.b,
                            0.72,
                        ),
                    ..Default::default()
                },
            );
        }

        if critical {
            let pulse = 0.08 + ((elapsed * 5.0).sin().abs() * 0.10);
            draw_rectangle(0.0, 0.0, width, height, Color::new(0.55, 0.0, 0.0, pulse));
            draw_rectangle_lines(12.0, 12.0, width - 24.0, height - 24.0, 2.0, Color::new(1.0, 0.10, 0.05, 0.72));
            draw_text_ex("!!! CRITICAL VESSEL CONDITION !!!", 390.0, 92.0, TextParams { font_size: 18, color: Color::new(1.0, 0.28, 0.18, 0.90), ..Default::default() });
        }

        // Very subtle refresh lines.
        let phase =
            (elapsed * 48.0)
                % 4.0;

        let mut y =
            phase - 4.0;

        while y < height {
            draw_rectangle(
                0.0,
                y,
                width,
                1.0,
                Color::new(
                    0.0,
                    0.0,
                    0.0,
                    0.11,
                ),
            );

            y += 4.0;
        }

        // Rare tiny flicker.
        if self.noise() > 0.997 {
            draw_rectangle(
                0.0,
                0.0,
                width,
                height,
                Color::new(
                    0.2,
                    0.9,
                    0.25,
                    0.035,
                ),
            );
        }

        // CRT edge darkening.
        let edge = 55.0;

        draw_rectangle(
            0.0,
            0.0,
            width,
            edge,
            Color::new(
                0.0,
                0.0,
                0.0,
                0.20,
            ),
        );

        draw_rectangle(
            0.0,
            height - edge,
            width,
            edge,
            Color::new(
                0.0,
                0.0,
                0.0,
                0.20,
            ),
        );

        draw_rectangle(
            0.0,
            0.0,
            edge,
            height,
            Color::new(
                0.0,
                0.0,
                0.0,
                0.16,
            ),
        );

        draw_rectangle(
            width - edge,
            0.0,
            edge,
            height,
            Color::new(
                0.0,
                0.0,
                0.0,
                0.16,
            ),
        );
    }
}
