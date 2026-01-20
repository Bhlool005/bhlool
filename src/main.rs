mod render;
mod scene;
mod shapes;

fn main() {
    let mut scene = scene::Scene::new();
    scene.add_rectangle(shapes::Rectangle::new(0, 0, 10, 5));
    scene.add_rectangle(shapes::Rectangle::new(3, 4, 2, 1));

    let output = render::render_scene(&scene);
    print!("{output}");
}

#[cfg(test)]
mod tests {
    use super::{render, scene, shapes};

    #[test]
    fn render_empty_scene() {
        let scene = scene::Scene::new();
        let output = render::render_scene(&scene);
        assert_eq!(output, "");
    }

    #[test]
    fn render_single_rectangle() {
        let mut scene = scene::Scene::new();
        scene.add_rectangle(shapes::Rectangle::new(0, 0, 10, 5));

        let output = render::render_scene(&scene);
        let expected = "rectangle x=0 y=0 width=10 height=5\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn render_multiple_rectangles() {
        let mut scene = scene::Scene::new();
        scene.add_rectangle(shapes::Rectangle::new(1, 2, 3, 4));
        scene.add_rectangle(shapes::Rectangle::new(-5, 0, 2, 2));

        let output = render::render_scene(&scene);
        let expected = concat!(
            "rectangle x=1 y=2 width=3 height=4\n",
            "rectangle x=-5 y=0 width=2 height=2\n"
        );
        assert_eq!(output, expected);
    }
}
