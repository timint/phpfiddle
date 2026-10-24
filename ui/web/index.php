<?php

  if (isset($_POST['code'])) {

    ini_set('display_errors', 'On');
    ini_set('display_startup_errors', 'On');

    if ($_POST['type'] == 'html') {
      ini_set('html_errors', 'On');
      header('Content-Type: text/html; charset=utf-8');
    } else {
      ini_set('html_errors', 'Off');
      header('Content-Type: text/plain; charset=utf-8');
    }

    $tmp_file = stream_get_meta_data(tmpfile())['uri'];
    register_shutdown_function('unlink', $tmp_file);

    file_put_contents($tmp_file, $_POST['code']);
    include $tmp_file;

    exit;
  }

?>
<link rel="stylesheet" href="style.css" />

<nav class="navbar">
  <div class="navbar-inner">
    <div href="/" class="brand"><img src="/images/php-logo.svg" width="48" height="24" alt="php"> <?php echo PHP_VERSION; ?></div>

    <form class="navbar-search" id="topsearch" target="_blank" action="https://www.php.net/search.php">
      <input type="search" name="pattern" class="search-query tt-query" placeholder="Search documentation" accesskey="s" autocomplete="off" spellcheck="false" dir="auto" />
    </form>

    <ul class="nav">
      <li><a href="https://www.php.net/manual/en/" target="_blank">Documentation</a></li>
      <li><a href="https://www.w3schools.com/php/" target="_blank">W3Schools</a></li>
    </ul>
  </div>
</nav>

<div class="row">
  <div class="col">
    <h2>Code</h2>
    <form name="code_form" action="/" method="post" target="result">

      <div style="overflow: hidden;">
        <div id="actions">
          <button type="button" name="clear">Clear</button>
          <button type="button" name="load">Load File</button>
          <button type="button" name="save">Save File</button>
        </div>
        <div id="output-type">
          Display output as:
          <label><input type="radio" name="type" value="text" checked />Text</label>
          <label><input type="radio" name="type" value="html" />HTML</label>
        </div>
      </div>
      <div id="editor"></div>
      <input type="hidden" name="code" />
      <button id="run" type="submit">Run (Ctrl + Return)</button>
    </form>
  </div>

  <div class="col">
    <h2>Result</h2>
    <iframe name="result" src="about:blank"></iframe>
  </div>
</div>

<link rel="stylesheet" href="assets/codemirror/lib/codemirror.css">
<link rel="stylesheet" href="assets/codemirror/theme/material.css">
<link rel="stylesheet" href="assets/codemirror/addon/scroll/simplescrollbars.css">
<script src="assets/codemirror/lib/codemirror.js"></script>
<script src="assets/codemirror/addon/edit/matchbrackets.js"></script>
<script src="assets/codemirror/mode/htmlmixed/htmlmixed.js"></script>
<script src="assets/codemirror/mode/xml/xml.js"></script>
<script src="assets/codemirror/mode/css/css.js"></script>
<script src="assets/codemirror/mode/clike/clike.js"></script>
<script src="assets/codemirror/addon/scroll/simplescrollbars.js"></script>
<script src="assets/codemirror/mode/php/php.js"></script>
<script>
  myCodeMirror = CodeMirror(document.querySelector('#editor'), {
    height: 'auto',
    lineNumbers: true,
    lineWrapping: true,
    matchBrackets: true,
    mode: "application/x-httpd-php",
    indentUnit: 2,
    indentWithTabs: false
  });

  myCodeMirror.setValue("<"+"?php\n\n$foo = 'bar';\nvar_dump($foo);");

  document.querySelector('form[name="code_form"]').addEventListener('submit', function(e){
    document.querySelector('iframe[name="result"]').contentWindow.document.write('<img src="images/loader.svg" />');
    document.querySelector('input[name="code"]').value = myCodeMirror.getValue();
  });

  document.body.addEventListener('keydown', function(e) {
    if(!(e.keyCode == 13 && (e.metaKey || e.ctrlKey))) return;
    document.querySelector('input[name="code"]').value = myCodeMirror.getValue();
    e.target.form.submit();
  });

  document.querySelector('button[name="clear"]').addEventListener('click', function(event){
    myCodeMirror.setValue("<"+"?php\n");
  });

  document.querySelector('button[name="load"]').addEventListener('click', function(event){

    let input = document.createElement('input');
    input.type = 'file';
    input.accept = '.php';
    input.onchange = _ => {
    // you can use this method to get file and perform respective operations
      let files = Array.from(input.files);
      var reader = new FileReader();
      reader.onload = function(event){
        myCodeMirror.setValue(event.target.result);
      };
      reader.readAsText(files[0]);
    };

    input.click();
  });

  document.querySelector('button[name="save"]').addEventListener('click', async function(event){
/*
  // ExeOutput for PHP 2001
    try {
      var file = new Blob([ myCodeMirror.getValue() ], {type: 'text/plain'});

      const handle = await showSaveFilePicker({
        suggestedName: 'script.php',
        types: [{
          description: 'PHP Script',
          accept: {
            'application/x-httpd-php': ['.php'],
          },
        }]
      });

      const writable = await handle.createWritable();
      await writable.write( file );
      writable.close();
    } catch (err) {
      alert('Error: ' + err.error)
    }
*/

    var textFileAsBlob = new Blob([ myCodeMirror.getValue() ], {type:'text/plain'});

    var downloadLink = document.createElement('a');
    downloadLink.download = "untitled.php";

    // hidden link title name
    downloadLink.innerHTML = 'LINKTITLE';

    window.URL = window.URL || window.webkitURL;

    downloadLink.href = window.URL.createObjectURL(textFileAsBlob);

    downloadLink.onclick = function(){
      this.remove();
    };

    downloadLink.style.display = 'none';
    document.body.appendChild(downloadLink);
    downloadLink.click();

  });
</script>