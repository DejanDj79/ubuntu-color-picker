use crate::color::Rgb;

pub fn load_css(provider: &gtk::CssProvider, color: Rgb) {
    let hex = color.hex();

    let text_color = if color.brightness() > 0.55 {
        "#111111"
    } else {
        "#F5F5F5"
    };

    let panel_color = if color.brightness() > 0.55 {
        "rgba(255,255,255,0.42)"
    } else {
        "rgba(0,0,0,0.28)"
    };

    let border_color = if color.brightness() > 0.55 {
        "rgba(0,0,0,0.18)"
    } else {
        "rgba(255,255,255,0.20)"
    };

    let css = format!(
        "
        window {{
            border-radius: 0;
        }}

        .app-bg {{
            background: {};
            color: {};
            border-radius: 0;
        }}

        .top-bar {{
            border-bottom: 1px solid {};
            padding-left: 10px;
            padding-right: 10px;
        }}

        .toolbar-button {{
            min-width: 34px;
            min-height: 34px;
            border-radius: 6px;
            background: transparent;
            color: {};
            border: none;
            padding: 0;
            box-shadow: none;
        }}

        .window-button {{
            min-width: 42px;
            min-height: 42px;
            border-radius: 0;
            background: transparent;
            color: {};
            border: none;
            padding: 0;
            box-shadow: none;
        }}

        .toolbar-button:hover {{
            min-width: 34px;
            min-height: 34px;
            border-radius: 6px;
            background: rgba(255,255,255,0.16);
        }}

        .window-button:hover {{
            min-width: 42px;
            min-height: 42px;
            border-radius: 0;
            background: rgba(255,255,255,0.16);
        }}

        .toolbar-button.favorite-active {{
            border-radius: 6px;
            background: rgba(255,255,255,0.22);
        }}

        .toolbar-button.palette-active {{
            border-radius: 6px;
            background: rgba(255,255,255,0.22);
        }}

        .channel-label {{
            font-weight: 700;
            color: {};
        }}

        .copy-toast {{
            background: rgba(0,0,0,0.72);
            border-radius: 7px;
            padding: 5px 10px;
        }}

        .copy-toast-label {{
            color: rgba(255,255,255,0.94);
            font-weight: 700;
            font-size: 11px;
        }}

        .copy-format-row {{
            border-radius: 0;
            background: transparent;
            padding: 8px 12px;
        }}

        .copy-format-row:hover {{
            background: rgba(128,128,128,0.16);
        }}

        entry {{
            border-radius: 0;
            background: {};
            color: {};
            border: 1px solid {};
            font-family: monospace;
            font-weight: 700;
        }}

        entry.input-error {{
            border-color: #ff4d4d;
            box-shadow: inset 0 0 0 1px rgba(255,77,77,0.55);
        }}

        .saved-colors-root {{
            min-width: 230px;
        }}

        .saved-section-label {{
            font-size: 11px;
            font-weight: 800;
            opacity: 0.75;
            margin-top: 4px;
        }}

        .saved-color-row {{
            min-height: 30px;
        }}

        .saved-color-button,
        .saved-color-copy,
        .saved-color-delete {{
            border-radius: 6px;
            background: transparent;
            padding: 4px 7px;
        }}

        .saved-color-button:hover,
        .saved-color-copy:hover,
        .saved-color-delete:hover {{
            background: rgba(128,128,128,0.16);
        }}

        .saved-empty-label {{
            font-size: 11px;
            opacity: 0.62;
            padding: 3px 7px;
        }}

        .palette-strip {{
            border-top: 1px solid {};
            padding-top: 7px;
            margin-top: 0;
        }}

        .palette-color-button {{
            border-radius: 999px;
            background: transparent;
            padding: 0;
        }}

        .palette-color-button:hover {{
            background: rgba(128,128,128,0.16);
        }}

        .palette-empty-label {{
            min-height: 24px;
        }}

        scale trough {{
            min-height: 8px;
            border-radius: 999px;
            background: {};
            border: 1px solid {};
        }}

        scale highlight {{
            border-radius: 999px;
            background: {};
        }}

        scale slider {{
            min-width: 18px;
            min-height: 18px;
            border-radius: 999px;
            background: {};
            border: 2px solid {};
        }}
        ",
        hex,
        text_color,
        border_color,
        text_color,
        text_color,
        text_color,
        panel_color,
        text_color,
        border_color,
        border_color,
        panel_color,
        border_color,
        text_color,
        text_color,
        border_color,
    );
    provider.load_from_data(&css);
}
