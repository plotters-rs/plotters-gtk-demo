use gtk::glib;

mod imp;

glib::wrapper! {
    pub struct GaussianPlot(ObjectSubclass<imp::GaussianPlot>)
        @extends gtk::Widget,
        @implements gtk::Buildable, gtk::ConstraintTarget, gtk::Accessible;
        
}
