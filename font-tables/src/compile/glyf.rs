struct SimpleGlyph {
    instructions: Vec<u8>,
    contours: Vec<Contour>,
}

struct Point {
    x: i16,
    y: i16,
    on_curve: bool,
}

struct Contour {
    points: Vec<Point>,
}

struct CompositeGlyph {
    components: Vec<Component>,
}

struct Component {
    glyph: GlyphId,
    position: Position,
    transform: Affine,
    flags: ComponentFlags,
}

enum Position {
    Vector { x: i16, y: i16 },
    Point { child: u16, parent: u16 }
}

enum Glyph {
    Simple(SimpleGlyph),
    Composite(CompositeGlyph),
}
