//! Render a file as an SVG.

use base64::prelude::{Engine as _, BASE64_STANDARD};
use std::{
    cell::Cell,
    env::args,
    io::{stdout, IsTerminal as _},
};
use svg::{
    node::{
        element::{Element, Group, Rectangle, Style, Text, Title, SVG},
        Comment, Text as TextNode,
    },
    Document, Node as _,
};

use cp437_tools::{
    internal::{process, ExitCode, Input, Output},
    prelude::{Meta, CP437_TO_UTF8},
};

#[allow(dead_code)]
#[must_use]
#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn main() -> ExitCode {
    return exec(&args().collect::<Vec<String>>());
}

#[inline]
#[must_use]
#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn exec(args: &[String]) -> ExitCode {
    let exit_code = if args.len() < 2 {
        ExitCode::USAGE(String::from("Missing input file"))
    } else if args.len() > 3 {
        ExitCode::USAGE(String::from("Too many arguments"))
    } else if stdout().is_terminal() {
        ExitCode::USAGE(String::from("Refusing to write to terminal"))
    } else {
        process(&args[1], |i, o| {
            return run(i, o, args.get(2).unwrap_or(&String::from("CLASSIC")));
        })
    };

    exit_code.print();
    return exit_code;
}

#[allow(missing_docs, reason = "Just an entry point")]
#[allow(clippy::missing_docs_in_private_items, reason = "Just an entry point")]
pub fn run(input: &mut Input, output: &mut Output, scheme: &String) -> ExitCode {
    let meta = input.meta.clone().unwrap_or(Meta { size: input.size, ..Default::default() });

    let (width, height) = meta.dimensions();
    let (width, height) = (width as usize, height as usize);
    let (font_width, font_height) = meta.font_size();
    let (font_width, font_height) = (font_width as usize, font_height as usize);
    let (ar_x, ar_y) = meta.aspect_ratio();
    let (ar_x, ar_y) = (ar_x as usize, ar_y as usize);
    let font_face = meta.font_face_woff();

    let mut document = prepare(input, (width, height), (font_width, font_height), (ar_x, ar_y), font_face);

    document = document.add(Comment::new("Drawing"));
    let drawing = Cell::new(
        Group::new().set("font-family", "IBM VGA").set("transform", format!("scale({ar_x}, {ar_y})")).add(
            Rectangle::new()
                .set("x", 0)
                .set("y", 0)
                .set("width", width * font_width)
                .set("height", height * font_height)
                .set("fill", "#000"),
        ),
    );

    input.read_by_bytes_full(
        |byte, (x, y), colour| {
            drawing.set(
                drawing
                    .take()
                    .add(
                        Rectangle::new()
                            .set("x", x as usize * font_width)
                            .set("y", y as usize * font_height)
                            .set("width", font_width)
                            .set("height", font_height)
                            .set("fill", format!("#{:02X}{:02X}{:02X}", colour[0][0], colour[0][1], colour[0][2])),
                    )
                    .add(
                        #[expect(clippy::integer_division, reason = "Intentional")]
                        Text::new(CP437_TO_UTF8[if byte > 0 { byte as usize } else { 32 }])
                            .set("x", x as usize * font_width)
                            .set("y", (y + 1) as usize * font_height - font_height / 4)
                            .set("font-size", font_height)
                            .set("fill", format!("#{:02X}{:02X}{:02X}", colour[1][0], colour[1][1], colour[1][2])),
                    ),
            );

            return Ok(());
        },
        scheme,
    )?;

    document = document.add(drawing.take());

    svg::write(output, &document)?;

    return ExitCode::OK;
}

/// Prepare the SVG with all corresponding metadata.
fn prepare(
    input: &mut Input,
    (width, height): (usize, usize),
    (font_width, font_height): (usize, usize),
    (ar_x, ar_y): (usize, usize),
    font_face: &[u8],
) -> SVG {
    let mut document = Document::new()
        .set(
            "viewBox",
            (0, 0, width * font_width * ar_x, height * font_height * ar_y),
        )
        .set("width", width * font_width * ar_x)
        .set("height", height * font_height * ar_y)
        .add(Comment::new("Embedded IBM VGA font, provided under CC-BY-SA-4.0"))
        .add(Comment::new("https://int10h.org/oldschool-pc-fonts"))
        .add(Style::new(format!(
            "@font-face {{ font-family: \"IBM VGA\"; src: url(\"data:application/font-woff;charset=utf-8;base64,{}\"); }}",
            BASE64_STANDARD.encode(font_face),
        )));

    if let Some(meta) = &input.meta {
        document = document.add(Comment::new("Metadata"));
        if let Some(title) = meta.title() {
            document = document.add(Title::new(title));
        }

        let mut description = Element::new("rdf:Description");
        description.assign("about", "");

        if let Some(title) = meta.title() {
            let mut title_elem = Element::new("dc:title");
            title_elem.append(TextNode::new(title.clone()));
            description.append(title_elem);
        }

        if let Some(author) = meta.author() {
            let mut creator_elem = Element::new("dc:creator");
            if let Some(group) = meta.group() {
                let mut author_elem = Element::new("rdf:li");
                author_elem.assign("dc:identifier", "author");
                author_elem.append(TextNode::new(author.clone()));
                let mut group_elem = Element::new("rdf:li");
                group_elem.assign("dc:identifier", "group");
                group_elem.append(TextNode::new(group.clone()));
                let mut bag = Element::new("rdf:Bag");
                bag.append(author_elem);
                bag.append(group_elem);
                creator_elem.append(bag);
            } else {
                creator_elem.assign("dc:identifier", "author");
                creator_elem.append(TextNode::new(author.clone()));
            }
            description.append(creator_elem);
        }

        if let Some(date) = meta.date() {
            let mut date_elem = Element::new("dc:date");
            date_elem.append(TextNode::new(format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])));
            description.append(date_elem);
        }

        let mut width_elem = Element::new("rdf:li");
        width_elem.assign("dc:identifier", "width");
        width_elem.append(TextNode::new(format!("{}", meta.width())));
        let mut height_elem = Element::new("rdf:li");
        height_elem.assign("dc:identifier", "height");
        height_elem.append(TextNode::new(format!("{}", meta.height())));
        let mut bag = Element::new("rdf:Bag");
        bag.append(width_elem);
        bag.append(height_elem);
        let mut format_elem = Element::new("dc:format");
        format_elem.assign("dc:identifier", "size");
        format_elem.append(bag);
        description.append(format_elem);

        if !meta.notes().is_empty() {
            let mut seq = Element::new("rdf:Seq");
            seq.assign("dc:identifier", "notes");
            for note in meta.notes() {
                let mut note_elem = Element::new("rdf:li");
                note_elem.append(TextNode::new(note.clone()));
                seq.append(note_elem);
            }
            description.append(seq);
        }

        let mut type_elem = Element::new("dc:type");
        type_elem.append(TextNode::new("http://purl.org/dc/dcmitype/StillImage"));
        description.append(type_elem);

        let mut rdf = Element::new("rdf:RDF");
        rdf.append(description);
        let mut metadata = Element::new("metadata");
        metadata.assign("xmlns:dc", "http://purl.org/dc/elements/1.1/");
        metadata.assign("xmlns:rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
        metadata.append(rdf);
        document = document.add(metadata);
    }

    return document;
}

#[path = "."]
#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[path = "../../libs/internal/test_utils.rs"]
    mod test;

    #[test]
    fn no_input() {
        assert_eq!(exec(&[String::from("cp437-to-svg")]), ExitCode::USAGE(String::from("Missing input file")));
    }

    #[test]
    fn too_many_args() {
        assert_eq!(
            exec(&[String::from("cp437-to-svg"), String::from("a"), String::from("b"), String::from("c")]),
            ExitCode::USAGE(String::from("Too many arguments")),
        );
    }

    #[ignore]
    #[test]
    fn stdout() {
        assert_eq!(
            exec(&[String::from("cp437-to-svg"), String::from("a")]),
            ExitCode::USAGE(String::from("Refusing to write to terminal")),
        );
    }

    #[test]
    fn simple() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/simple.ans",
            "res/test/simple.svg",
        );
    }

    #[test]
    fn meta() -> Result<(), String> {
        return test::file(|i, o| return run(i, o, &String::from("CLASSIC")), "res/test/meta.ans", "res/test/meta.svg");
    }

    #[test]
    fn notes() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/comments.ans",
            "res/test/comments.svg",
        );
    }

    #[test]
    fn background() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/test/background.ans",
            "res/test/background.svg",
        );
    }

    #[test]
    fn logo() -> Result<(), String> {
        return test::file(|i, o| return run(i, o, &String::from("CLASSIC")), "res/logo/logo.ans", "res/logo/logo.svg");
    }

    #[test]
    fn banner() -> Result<(), String> {
        return test::file(
            |i, o| return run(i, o, &String::from("CLASSIC")),
            "res/banner/banner.ans",
            "res/banner/banner.svg",
        );
    }
}
