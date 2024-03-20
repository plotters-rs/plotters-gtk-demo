use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use std::cell::Cell;
use std::error::Error;
use std::f64;

use plotters::prelude::*;
use plotters_cairo::CairoBackend;

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::GaussianPlot)]
pub struct GaussianPlot {
    #[property(get, set, minimum = -f64::consts::PI, maximum = f64::consts::PI)]
    pitch: Cell<f64>,
    #[property(get, set, minimum = 0.0, maximum = f64::consts::PI)]
    yaw: Cell<f64>,
    #[property(get, set, minimum = -10.0, maximum = 10.0)]
    mean_x: Cell<f64>,
    #[property(get, set, minimum = -10.0, maximum = 10.0)]
    mean_y: Cell<f64>,
    #[property(get, set, minimum = 0.0, maximum = 10.0)]
    std_x: Cell<f64>,
    #[property(get, set, minimum = 0.0, maximum = 10.0)]
    std_y: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for GaussianPlot {
    const NAME: &'static str = "GaussianPlot";
    type Type = super::GaussianPlot;
    type ParentType = gtk::Widget;
}

impl ObjectImpl for GaussianPlot {
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        Self::derived_set_property(self, id, value, pspec);
        self.obj().queue_draw();
    }

    fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        Self::derived_property(self, id, pspec)
    }
}

impl WidgetImpl for GaussianPlot {
    fn snapshot(&self, snapshot: &gtk::Snapshot) {
        let width = self.obj().width() as u32;
        let height = self.obj().height() as u32;
        if width == 0 || height == 0 {
            return;
        }

        let bounds = gtk::graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
        let cr = snapshot.append_cairo(&bounds);
        let backend = CairoBackend::new(&cr, (width, height)).unwrap();
        self.plot_pdf(backend).unwrap();
    }
}

impl GaussianPlot {
    fn gaussian_pdf(&self, x: f64, y: f64) -> f64 {
        let x_diff = (x - self.mean_x.get()) / self.std_x.get();
        let y_diff = (y - self.mean_y.get()) / self.std_y.get();
        let exponent = -(x_diff * x_diff + y_diff * y_diff) / 2.0;
        let denom = (2.0 * std::f64::consts::PI / self.std_x.get() / self.std_y.get()).sqrt();
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
            p.pitch = self.pitch.get();
            p.yaw = self.yaw.get();
            p.scale = 0.7;
            p.into_matrix() // build the projection matrix
        });

        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .max_light_lines(3)
            .draw()?;
        chart.draw_series(
            SurfaceSeries::xoz(
                (-50..=50).map(|x| x as f64 / 5.0),
                (-50..=50).map(|x| x as f64 / 5.0),
                |x, y| self.gaussian_pdf(x, y),
            )
            .style_func(&|&v| (&HSLColor(240.0 / 360.0 - 240.0 / 360.0 * v, 1.0, 0.7)).into()),
        )?;

        root.present()?;
        Ok(())
    }
}
