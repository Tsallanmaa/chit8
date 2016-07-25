use sdl2;
use sdl2::pixels::Color;

pub trait Display 
{

}

#[allow(dead_code)] // TODO: Remove when this is actually used
pub struct SdlDisplay<'a>
{
	renderer: Box<sdl2::render::Renderer<'a>>
}

impl<'a> SdlDisplay<'a>
{
	pub fn new(sdl_context: sdl2::Sdl) -> SdlDisplay<'a>
	{
	    let video_subsystem = sdl_context.video().unwrap();

	    let window = video_subsystem.window("CHIT8", 800, 600)
	        .position_centered()
	        .build()
	        .unwrap();

	    let mut renderer = window.renderer().build().unwrap();

	    renderer.set_draw_color(Color::RGB(255, 0, 0));
	    renderer.clear();
	    renderer.present();	

	    SdlDisplay { renderer: Box::new(renderer) }	
	}
}

impl<'a> Display for SdlDisplay<'a>
{

}