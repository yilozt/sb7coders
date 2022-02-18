// For more comments about what's going on here, check out the `hello_world`
// example.
import('./pkg')
  .then(app => {
    let inner_html = [
      '_default',
      '_ch2_main',
      '_ch3_1_vertexattr',
      '_ch3_2_transdata'
    ].map(demo => `<option value="${demo}">${demo.slice(1)}</option>`)
    .join('\n');

    /** @type HTMLSelectElement */
    let select = document.getElementById('select');
    select.innerHTML = inner_html;

    const load = (demo) => {
      app[old_val].stop();
      app[demo].run();
      old_val = demo;  
    }

    let old_val = select.value = '_default';
    select.onchange = (e) => {
      if (old_val !== e.value) {
        load(select.value);
      }
    }

    app[old_val].run();
  })
  .catch(console.error)
