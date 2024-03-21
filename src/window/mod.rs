mod imp;

use glib::prelude::*;
use gtk::glib;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget;
}

impl Window {
    pub fn new<P: IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
