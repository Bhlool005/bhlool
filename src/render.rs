use crate::scene::Scene;

pub fn render_scene(scene: &Scene) -> String {
    let mut output = String::new();

    for rectangle in scene.rectangles() {
        output.push_str(&format!(
            "rectangle x={} y={} width={} height={}\n",
            rectangle.x, rectangle.y, rectangle.width, rectangle.height
        ));
    }

    output
}
