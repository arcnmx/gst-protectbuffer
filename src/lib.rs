use std::sync::RwLock;
use glib::subclass::{self, prelude::*};
use glib::{Cast, ToValue};
use gstreamer::{Plugin, Element, Caps, PadTemplate, PadPresence, PadDirection, Buffer, BufferRef, MemoryFlags, Rank, FlowSuccess, FlowError, gst_error};
use gstreamer::subclass::prelude::*;
use gstreamer_base::BaseTransform;
use gstreamer_base::subclass::prelude::*;
use gstreamer_base::subclass::base_transform::PrepareOutputBufferSuccess;
use once_cell::sync::Lazy;

#[derive(Debug, Default)]
struct Settings {
	copy: bool,
}

#[derive(Default)]
pub struct BufferProtect {
	settings: RwLock<Settings>,
}

fn plugin_init(plugin: &Plugin) -> Result<(), glib::BoolError> {
	Element::register(Some(plugin),
		"protectbuffer", Rank::None, BufferProtect::get_type())?;

	Ok(())
}

gstreamer::gst_plugin_define! {
	protectbuffer,
	env!("CARGO_PKG_DESCRIPTION"),
	plugin_init,
	env!("CARGO_PKG_VERSION"),
	"MIT/X11",
	env!("CARGO_PKG_NAME"),
	env!("CARGO_PKG_NAME"),
	env!("CARGO_PKG_REPOSITORY"),
	"2000-01-01"
}

impl ObjectSubclass for BufferProtect {
	const NAME: &'static str = "GstBufferProtect";
	type ParentType = BaseTransform;
	type Instance = gstreamer::subclass::ElementInstanceStruct<Self>;
	type Class = subclass::simple::ClassStruct<Self>;

	glib::glib_object_subclass!();

	fn class_init(klass: &mut Self::Class) {
		klass.set_metadata(
			"Buffer Protection",
			"Filters/Whatever",
			env!("CARGO_PKG_DESCRIPTION"),
			env!("CARGO_PKG_AUTHORS"),
		);

		let caps = Caps::new_any();

		klass.add_pad_template(PadTemplate::new(
			"src",
			PadDirection::Src,
			PadPresence::Always,
			&caps,
		).unwrap());

		klass.add_pad_template(PadTemplate::new(
			"sink",
			PadDirection::Sink,
			PadPresence::Always,
			&caps,
		).unwrap());

		klass.install_properties(&PROPERTIES);

		klass.configure(
			gstreamer_base::subclass::BaseTransformMode::AlwaysInPlace,
			true,
			true,
		);
	}

	fn new() -> Self {
		Self::default()
	}
}

static PROPERTIES: [subclass::Property; 1] = [
	subclass::Property("copy", |_| glib::ParamSpec::boolean("copy", "Copy", "Copy buffer contents", false, glib::ParamFlags::READWRITE /*| gstreamer::PARAM_FLAG_MUTABLE_PLAYING*/))
];

static CAT: Lazy<gstreamer::DebugCategory> = Lazy::new(|| {
	gstreamer::DebugCategory::new(
		"protectbuffer",
		gstreamer::DebugColorFlags::empty(),
		Some("Buffer Protection"),
	)
});

impl ObjectImpl for BufferProtect {
	glib::glib_object_impl!();

	fn set_property(&self, obj: &glib::Object, id: usize, value: &glib::Value) {
		match id {
			0 => {
				let mut settings = self.settings.write().unwrap();
				settings.copy = value.get_some().expect("type checked upstream");
			},
			_ => {
				let element = obj.downcast_ref::<BaseTransform>().unwrap();
				gst_error!(
					CAT,
					obj: element,
					"unknown set_property({})", id
				);
			},
		}
	}

	fn get_property(&self, obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
		match id {
			0 => {
				let value = {
					let settings = self.settings.read().unwrap();
					settings.copy
				};
				Ok(value.to_value())
			},
			_ => {
				let element = obj.downcast_ref::<BaseTransform>().unwrap();
				gst_error!(
					CAT,
					obj: element,
					"unknown get_property({})", id
				);
				Err(())
			},
		}
	}
}

impl ElementImpl for BufferProtect {

}

impl BaseTransformImpl for BufferProtect {
	fn prepare_output_buffer(&self, element: &BaseTransform, inbuf: &BufferRef) -> Result<PrepareOutputBufferSuccess, FlowError> {
		let copy = self.settings.read().unwrap().copy;
		Ok(match !copy && inbuf.iter_memories().all(|mem| mem.get_flags().intersects(MemoryFlags::READONLY | MemoryFlags::NOT_MAPPABLE)) {
			true => PrepareOutputBufferSuccess::InputBuffer,
			_ => {
				let mut buffer = Buffer::new();
				let bufref = buffer.make_mut();
				let flags = match copy {
					true => gstreamer::BUFFER_COPY_ALL | gstreamer::BufferCopyFlags::DEEP,
					false => gstreamer::BUFFER_COPY_METADATA,
				};
				inbuf.copy_into(bufref, flags, 0, None).map_err(|e| {
					gst_error!(CAT,
						obj: element,
						"Failed to copy output buffer: {:?}", e);
					FlowError::Error
				})?;
				if !copy {
					for mem in inbuf.iter_memories() {
						bufref.append_memory(match mem.get_flags().contains(MemoryFlags::NO_SHARE) {
							false => mem.share(0, None),
							_ => mem.copy(),
						});
					}
				}
				PrepareOutputBufferSuccess::Buffer(buffer)
			},
		})
	}

	fn transform_ip_passthrough(&self, _element: &BaseTransform, _buf: &Buffer) -> Result<FlowSuccess, FlowError> {
		Ok(FlowSuccess::Ok)
	}
}
