var htmlEditor = ace.edit('html');
htmlEditor.setTheme("ace/theme/monokai");
htmlEditor.session.setMode("ace/mode/html");


var cssEditor = ace.edit('css');
cssEditor.setTheme("ace/theme/monokai");
cssEditor.session.setMode("ace/mode/css");

htmlEditor.setFontSize(15);
cssEditor.setFontSize(15);

const update = () => {
  document.getElementById('preview').innerHTML = htmlEditor.getValue()+parseStyle(cssEditor.getValue());
}
const parseStyle = (css) => {
  // adds style tags and makes sure css only applies to html inside preview
  let newCSS = '<style>';
  css=css.split("\n")
  for (var line of css) {
    if (line.includes('{')) {
      line = '#preview '+line;
    }
    newCSS += line;
  }
  return newCSS+'</style>';
}
window.onload = () => update();
window.onkeyup = () => update();


// set default template
htmlEditor.setValue(`<html>
<body>
  <h1>Hello, World</h1>
  <p>Lorem ipsum...</p>
</body>
</html>`);
cssEditor.setValue(`h1 {
  color: red;
  margin: 0;
}`);