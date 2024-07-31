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
