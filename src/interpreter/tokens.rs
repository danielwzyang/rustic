#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
    pub info: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Command(Function),
    AxisOfRotation,
    Number,
    FilePath,
    Identifier,
    EasingFunction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Function {
    // GENERAL
    Display,
    Save,
    Clear,
    SetCamera,

    // TRANSFORMATIONS
    Push,
    Pop,
    Move,
    Scale,
    Rotate,

    // EDGES
    Line,
    Circle,
    Hermite,
    Bezier,

    // POLYGONS
    Polygon,
    Box,
    Sphere,
    Torus,
    Cylinder,
    Mesh,

    // LIGHTING
    AddLight,
    ClearLights,
    SetAmbient,
    DefineConstants,
    SetShading,

    // ANIMATION
    SetBaseName,
    SetKnob,
    SaveKnobList,
    Tween,
    SetFrames,
    VaryKnob,
    SetAllKnobs,

    // UNIMPLEMENTED BUT RECOGNIZED
    SaveCoordSystem,
    GenerateRayFiles,
    SetFocalLength,
}
