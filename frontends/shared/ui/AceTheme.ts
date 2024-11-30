/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-explicit-any */
(window as any).ace.define(
  "ace/theme/trieve",
  ["require", "exports", "module", "ace/lib/dom"],
  (acequire: any, exports: any, _module: any) => {
    exports.isDark = true;
    exports.cssClass = "ace-jsoneditor";
    exports.cssText = `.ace-jsoneditor .ace_gutter {
  background: #1a0005;
  color: steelblue
}

.ace-jsoneditor .ace_print-margin {
  width: 1px;
  background: #1a1a1a
}

.ace-jsoneditor {
  background-color: black;
  color: #DEDEDE
}

.ace-jsoneditor .ace_cursor {
  color: #9F9F9F
}

.ace-jsoneditor .ace_marker-layer .ace_selection {
  background: #424242
}

.ace-jsoneditor.ace_multiselect .ace_selection.ace_start {
  box-shadow: 0 0 3px 0px black;
}

.ace-jsoneditor .ace_marker-layer .ace_step {
  background: rgb(0, 0, 0)
}

.ace-jsoneditor .ace_marker-layer .ace_bracket {
  background: #090;
}

.ace-jsoneditor .ace_marker-layer .ace_bracket-start {
  background: #090;
}

.ace-jsoneditor .ace_marker-layer .ace_bracket-unmatched {
  margin: -1px 0 0 -1px;
  border: 1px solid #900
}

.ace-jsoneditor .ace_marker-layer .ace_active-line {
  background: #2A2A2A
}

.ace-jsoneditor .ace_gutter-active-line {
  background-color: #000000
}

.ace-jsoneditor .ace_marker-layer .ace_selected-word {
  border: 1px solid #424242
}

.ace-jsoneditor .ace_invisible {
  color: #343434
}

.ace-jsoneditor .ace_keyword,
.ace-jsoneditor .ace_meta,
.ace-jsoneditor .ace_storage,
.ace-jsoneditor .ace_storage.ace_type,
.ace-jsoneditor .ace_support.ace_type {
  color: tomato
}

.ace-jsoneditor .ace_keyword.ace_operator {
  color: deeppink
}

.ace-jsoneditor .ace_constant.ace_character,
.ace-jsoneditor .ace_constant.ace_language,
.ace-jsoneditor .ace_constant.ace_numeric,
.ace-jsoneditor .ace_keyword.ace_other.ace_unit,
.ace-jsoneditor .ace_support.ace_constant,
.ace-jsoneditor .ace_variable.ace_parameter {
  color: #E78C45
}

.ace-jsoneditor .ace_constant.ace_other {
  color: gold
}

.ace-jsoneditor .ace_invalid {
  color: yellow;
  background-color: red
}

.ace-jsoneditor .ace_invalid.ace_deprecated {
  color: #CED2CF;
  background-color: #B798BF
}

.ace-jsoneditor .ace_fold {
  background-color: #7AA6DA;
  border-color: #DEDEDE
}

.ace-jsoneditor .ace_entity.ace_name.ace_function,
.ace-jsoneditor .ace_support.ace_function,
.ace-jsoneditor .ace_variable {
  color: #7AA6DA
}

.ace-jsoneditor .ace_support.ace_class,
.ace-jsoneditor .ace_support.ace_type {
  color: #E7C547
}

.ace-jsoneditor .ace_heading,
.ace-jsoneditor .ace_string {
  color: #B9CA4A
}

.ace-jsoneditor .ace_entity.ace_name.ace_tag,
.ace-jsoneditor .ace_entity.ace_other.ace_attribute-name,
.ace-jsoneditor .ace_meta.ace_tag,
.ace-jsoneditor .ace_string.ace_regexp,
.ace-jsoneditor .ace_variable {
  color: #D54E53
}

.ace-jsoneditor .ace_comment {
  color: orangered
}

.ace-jsoneditor .ace_indent-guide {
  background: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAACCAYAAACZgbYnAAAAEklEQVQImWNgYGBgYLBWV/8PAAK4AYnhiq+xAAAAAElFTkSuQmCC) right repeat-y;
}

.ace-jsoneditor .ace_indent-guide-active {
  background: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAACCAYAAACZgbYnAAAAEklEQVQIW2PQ1dX9zzBz5sz/ABCcBFFentLlAAAAAElFTkSuQmCC) right repeat-y;
}`;

    const dom = acequire("../lib/dom");
    dom.importCssString(exports.cssText, exports.cssClass);
  },
);

(window as any).ace.define(
  "ace/theme/trieve-light",
  ["require", "exports", "module", "ace/lib/dom"],
  (acequire: any, exports: any, _module: any) => {
    exports.isDark = true;
    exports.cssClass = "ace-jsoneditor";
    exports.cssText = `.ace-jsoneditor .ace_gutter {
  background: #ffffff;
  color: #D4D4D4
}

.ace-jsoneditor .ace_print-margin {
    width: 1px;
    background: #e8e8e8;
}

.ace-jsoneditor {
    background-color: #FFFFFF;
    color: #24292E;
}

.ace-jsoneditor .ace_cursor {
    color: #044289;
    background: none;
}

.ace-jsoneditor .ace_marker-layer .ace_selection {
    background: rgba(3, 102, 214, 0.14);
}

.ace-jsoneditor.ace_multiselect .ace_selection.ace_start {
    box-shadow: 0 0 3px 0px #FFFFFF;
    border-radius: 2px;
}

.ace-jsoneditor .ace_marker-layer .ace_step {
    background: rgb(198, 219, 174);
}

.ace-jsoneditor .ace_marker-layer .ace_bracket {
    margin: -1px 0 0 -1px;
    border: 1px solid rgba(52, 208, 88, 0);
    background: rgba(52, 208, 88, 0.25);
}

.ace-jsoneditor .ace_marker-layer .ace_active-line {
    background: #f6f8fa;
    border: 2px solid #eeeeee;
}

.ace-jsoneditor .ace_gutter-active-line {
    background-color: #f6f8fa;
    color: #24292e
}

.ace-jsoneditor .ace_marker-layer .ace_selected-word {
    border: 1px solid rgba(3, 102, 214, 0.14);
}

.ace-jsoneditor .ace_fold {
    background-color: #D73A49;
    border-color: #24292E;
}

.ace_tooltip.ace-jsoneditor {
    background-color: #f6f8fa !important;
    color: #444d56 !important;
    border: 1px solid #444d56
}

.ace-jsoneditor .language_highlight_error {
    border-bottom: dotted 1px #cb2431;
    background: none;
}

.ace-jsoneditor .language_highlight_warning {
    border-bottom: solid 1px #f9c513;
    background: none;
}

.ace-jsoneditor .language_highlight_info {
    border-bottom: dotted 1px #1a85ff;
    background: none;
}

.ace-jsoneditor .ace_keyword {
    color: #D73A49;
}

.ace-jsoneditor .ace_constant {
    color: #005CC5;
}

.ace-jsoneditor .ace_support {
    color: #005CC5;
}

.ace-jsoneditor .ace_support.ace_constant {
    color: #005CC5;
}

.ace-jsoneditor .ace_support.ace_type {
    color: #D73A49;
}

.ace-jsoneditor .ace_storage {
    color: #D73A49;
}

.ace-jsoneditor .ace_storage.ace_type {
    color: #D73A49;
}

.ace-jsoneditor .ace_invalid.ace_illegal {
    font-style: italic;
    color: #B31D28;
}

.ace-jsoneditor .ace_invalid.ace_deprecated {
    font-style: italic;
    color: #B31D28;
}

.ace-jsoneditor .ace_string {
    color: #032F62;
}

.ace-jsoneditor .ace_string.ace_regexp {
    color: #032F62;
}

.ace-jsoneditor .ace_comment {
    color: #6A737D;
}

.ace-jsoneditor .ace_variable {
    color: #E36209;
}

.ace-jsoneditor .ace_variable.ace_language {
    color: #005CC5;
}

.ace-jsoneditor .ace_entity.ace_name {
    color: #6F42C1;
}

.ace-jsoneditor .ace_entity {
    color: #6F42C1;
}

.ace-jsoneditor .ace_entity.ace_name.ace_tag {
    color: #22863A;
}

.ace-jsoneditor .ace_meta.ace_tag {
    color: #22863A;
}

.ace-jsoneditor .ace_markup.ace_heading {
    color: #005CC5;
}

.ace-jsoneditor .ace_indent-guide {
  background: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAACCAYAAACZgbYnAAAAE0lEQVQImWP4////f4bLly//BwAmVgd1/w11/gAAAABJRU5ErkJggg==") right repeat-y;
}

.ace-jsoneditor .ace_indent-guide-active {
  background: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAACCAYAAACZgbYnAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAIGNIUk0AAHolAACAgwAA+f8AAIDpAAB1MAAA6mAAADqYAAAXb5JfxUYAAAAZSURBVHjaYvj///9/hivKyv8BAAAA//8DACLqBhbvk+/eAAAAAElFTkSuQmCC") right repeat-y;
}`;

    const dom = acequire("../lib/dom");
    dom.importCssString(exports.cssText, exports.cssClass);
  },
);
