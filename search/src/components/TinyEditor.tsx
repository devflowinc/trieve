/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
import { createEffect, onCleanup } from "solid-js";

interface TinyEditorProps {
  htmlValue?: string;
  onHtmlChange?: (value: string) => void;
  onTextChange?: (value: string) => void;
}

export const TinyEditor = (props: TinyEditorProps) => {
  let tinyEditor;

  const initEditor = async () => {
    const options = {
      selector: "#editor",
      height: "100%",
      width: "100%",
      plugins: [
        "advlist",
        "autoresize",
        "autolink",
        "lists",
        "link",
        "image",
        "charmap",
        "preview",
        "anchor",
        "searchreplace",
        "visualblocks",
        "code",
        "fullscreen",
        "insertdatetime",
        "media",
        "table",
        "help",
        "wordcount",
      ],
      autoresize_bottom_margin: 0,
      skin: document.documentElement.classList.contains("dark")
        ? "oxide-dark"
        : "oxide",
      content_css: document.documentElement.classList.contains("dark")
        ? ["dark"]
        : ["default"],
      toolbar:
        "undo redo | fontsize | " +
        "bold italic backcolor | alignleft aligncenter " +
        "alignright alignjustify | bullist numlist outdent indent | " +
        "removeformat | help",
      font_size_formats: "4pt 6pt 8pt 10pt 12pt 14pt 16pt 18pt 20pt 22pt",
      content_style:
        "body { font-family:Helvetica,Arial,sans-serif; font-size:12pt; min-height: 200px; }",
      menubar: false,
      entity_encoding: "raw",
      entities: "160,nbsp,38,amp,60,lt,62,gt",
      // eslint-disable-next-line
      setup: function (editor: any) {
        //eslint-disable-next-line
        editor.on("change keyup", () => {
          if (props.onHtmlChange) {
            props.onHtmlChange(editor.getContent());
          }
          if (props.onTextChange) {
            props.onTextChange(editor.getBody().textContent);
          }
        });
        editor.addShortcut("meta+shift+1", "Font size 8.", function () {
          editor.execCommand("FontSize", false, `8pt`);
        });

        editor.addShortcut("meta+shift+2", "Font size 12.", function () {
          editor.execCommand("FontSize", false, `12pt`);
        });

        editor.addShortcut("meta+shift+3", "Font size 16.", function () {
          editor.execCommand("FontSize", false, `16pt`);
        });

        editor.addShortcut("meta+shift+4", "Font size 20.", function () {
          editor.execCommand("FontSize", false, `20pt`);
        });

        editor.addShortcut("meta+shift+5", "Font size 24.", function () {
          editor.execCommand("FontSize", false, `24pt`);
        });

        editor.addShortcut("meta+shift+h", "Highlight color.", function () {
          editor.execCommand("HiliteColor", false, `#F1C40F`);
        });
      },
    };

    try {
      // eslint-disable-next-line
      const tinyMCE = (window as any).tinymce;
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      await tinyMCE.init(options);
      if (props.htmlValue) {
        // eslint-disable-next-line
        tinyMCE.get("editor").setContent(props.htmlValue);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const destroyEditor = () => {
    const tinyMCE = (window as any).tinymce; //eslint-disable-line
    if (tinyMCE) {
      const editor = tinyMCE.get("editor");
      if (editor) {
        editor.remove();
      }
    }
  };

  createEffect(() => {
    void initEditor();
    onCleanup(() => {
      destroyEditor();
    });
  });

  return <textarea ref={tinyEditor} id="editor" />;
};
