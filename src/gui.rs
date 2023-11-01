use crate::Screen;
use egui::{ComboBox, Context};
use egui_wgpu::{winit::Painter, WgpuConfiguration};
use egui_winit::State;
use ndi::Source;
use winit::event::WindowEvent;
use winit::window::Window;

pub fn update(screen: &mut Screen, sources: &[Source]) {
    if screen.ui_open {
        let ctx = screen.egui.begin_frame();
        egui::Window::new("Settings").show(&ctx, |ui| {
            ui.label("Sources");
            ComboBox::from_label("Source")
                .selected_text(if let Some(ref source) = screen.selected_source {
                    source.get_name()
                } else {
                    "Select a source".to_string()
                })
                .show_ui(ui, |ui| {
                    let selected = screen
                        .selected_source
                        .as_ref()
                        .map(|source| source.get_name());
                    for source in sources {
                        let name = source.get_name();
                        if ui
                            .add(egui::SelectableLabel::new(
                                selected.as_ref() == Some(&name),
                                &name,
                            ))
                            .clicked()
                        {
                            screen.selected_source = Some(source.clone());
                            screen.selected_source_send.send(source.clone()).unwrap();
                        }
                    }
                });
        });
        screen.egui.end_frame(&screen.window);
    }
}

pub struct Egui {
    context: Context,
    painter: Painter,
    state: State,
}

impl Egui {
    pub async fn from_window(window: &Window) -> color_eyre::Result<Self> {
        let context = Context::default();

        let mut painter = Painter::new(WgpuConfiguration::default(), 1, None, false);
        painter.set_window(Some(window)).await?;

        let state = State::new(window);

        Ok(Self {
            context,
            painter,
            state,
        })
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        let _ = self.state.on_event(&self.context, event);
        match event {
            WindowEvent::Resized(size) => {
                self.painter.on_window_resized(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.painter
                    .on_window_resized(new_inner_size.width, new_inner_size.height);
            }
            _ => {}
        }
    }

    fn begin_frame(&self) -> &Context {
        self.context.begin_frame(Default::default());

        &self.context
    }

    fn end_frame(&mut self, window: &Window) {
        let output = self.context.end_frame();
        self.state
            .handle_platform_output(window, &self.context, output.platform_output);

        let clipped_primitives = self.context.tessellate(output.shapes);

        self.painter.paint_and_update_textures(
            self.context.pixels_per_point(),
            [0., 0., 0., 1.],
            &clipped_primitives,
            &output.textures_delta,
            false,
        );
    }
}
