// For more comments about what's going on here, check out the `hello_world`
// example.
import('./pkg')
  .then(app => {
    const demos = [
      '_default',
      '_ch3_1_vertexattr',
      '_ch3_2_transdata',
      '_ch5_1_vao',
      '_ch5_2_spinningcube',
      '_ch5_3_spinningcubes',
      '_ch5_4_simpletexture',
      '_ch5_5_simpletexcoords',
      '_ch5_6_texturefilter'
    ];

    let inner_html = demos.map(demo => `<option value="${demo}">${demo.slice(1)}</option>`)
    .join('\n');

    app[demos[demos.length - 1]].run();

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
  })
  .catch(console.error)
