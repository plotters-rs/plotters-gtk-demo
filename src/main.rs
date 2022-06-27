use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;

const GLADE_UI_SOURCE: &'static str = include_str!("ui.glade");

#[derive(Clone, Copy)]
struct PlottingState {
    mean_x: f64,
    mean_y: f64,
    std_x: f64,
    std_y: f64,
    pitch: f64,
    roll: f64,
}

impl PlottingState {
    fn guassian_pdf(&self, x: f64, y: f64) -> f64 {
        let x_diff = (x - self.mean_x) / self.std_x;
        let y_diff = (y - self.mean_y) / self.std_y;
        let exponent = -(x_diff * x_diff + y_diff * y_diff) / 2.0;
        let denom = (2.0 * std::f64::consts::PI / self.std_x / self.std_y).sqrt();
        let gaussian_pdf = 1.0 / denom;
        gaussian_pdf * exponent.exp()
    }
    fn plot_pdf<'a, DB: DrawingBackend + 'a>(
        &self,
        backend: DB,
    ) -> Result<(), Box<dyn Error + 'a>> {
        let root = backend.into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root).build_cartesian_3d(
            -10.0f64..10.0,
            0.0f64..1.2,
            -10.0f64..10.0,
        )?;

        chart.with_projection(|mut p| {
            p.pitch = self.pitch;
            p.yaw = self.roll;
            p.scale = 0.7;
            p.into_matrix() // build the projection matrix
        });

        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .max_light_lines(3)
            .draw()?;
        let self_cloned = self.clone();
        chart.draw_series(
            SurfaceSeries::xoz(
                (-50..=50).map(|x| x as f64 / 5.0),
                (-50..=50).map(|x| x as f64 / 5.0),
                move |x, y| self_cloned.guassian_pdf(x, y),
            )
            .style_func(&|&v| (&HSLColor(240.0 / 360.0 - 240.0 / 360.0 * v, 1.0, 0.7)).into()),
        )?;

        root.present()?;
        Ok(())
    }
}

fn build_ui(app: &gtk::Application) {
    let builder = gtk::Builder::from_string(GLADE_UI_SOURCE);
    let window = builder.object::<gtk::Window>("MainWindow").unwrap();

    window.set_title("Gaussian PDF Plotter");

    let drawing_area: gtk::DrawingArea = builder.object("MainDrawingArea").unwrap();
    let pitch_scale = builder.object::<gtk::Scale>("PitchScale").unwrap();
    let yaw_scale = builder.object::<gtk::Scale>("YawScale").unwrap();
    let mean_x_scale = builder.object::<gtk::Scale>("MeanXScale").unwrap();
    let mean_y_scale = builder.object::<gtk::Scale>("MeanYScale").unwrap();
    let std_x_scale = builder.object::<gtk::Scale>("SDXScale").unwrap();
    let std_y_scale = builder.object::<gtk::Scale>("SDYScale").unwrap();

    let app_state = Rc::new(RefCell::new(PlottingState {
        mean_x: mean_x_scale.value(),
        mean_y: mean_y_scale.value(),
        std_x: std_x_scale.value(),
        std_y: std_y_scale.value(),
        pitch: pitch_scale.value(),
        roll: yaw_scale.value(),
    }));

    window.set_application(Some(app));

    let state_cloned = app_state.clone();
    drawing_area.connect_draw(move |widget, cr| {
        let state = state_cloned.borrow().clone();
        let w = widget.allocated_width();
        let h = widget.allocated_height();
        let backend = CairoBackend::new(cr, (w as u32, h as u32)).unwrap();
        state.plot_pdf(backend).unwrap();
        Inhibit(false)
    });

    let handle_change =
        |what: &gtk::Scale, how: Box<dyn Fn(&mut PlottingState) -> &mut f64 + 'static>| {
            let app_state = app_state.clone();
            let drawing_area = drawing_area.clone();
            what.connect_value_changed(move |target| {
                let mut state = app_state.borrow_mut();
                *how(&mut *state) = target.value();
                drawing_area.queue_draw();
            });
        };

    handle_change(&pitch_scale, Box::new(|s| &mut s.pitch));
    handle_change(&yaw_scale, Box::new(|s| &mut s.roll));
    handle_change(&mean_x_scale, Box::new(|s| &mut s.mean_x));
    handle_change(&mean_y_scale, Box::new(|s| &mut s.mean_y));
    handle_change(&std_x_scale, Box::new(|s| &mut s.std_x));
    handle_change(&std_y_scale, Box::new(|s| &mut s.std_y));

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("io.github.plotters-rs.plotters-gtk-demo"),
        Default::default(),
    );

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
