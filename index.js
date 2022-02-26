// For more comments about what's going on here, check out the `hello_world`

// example.
import('./pkg/sb7')
  .then(async app => {
    const demos = Object.keys(app).filter(v => v.match(/^_c/));
    demos.sort((a, b) => {
      let [cha, numa] = a.slice(3).split('_');
      let [chb, numb] = b.slice(3).split('_');
      return (Number(cha) * 100 + Number(numa)) > (Number(chb) * 100 + Number(numb))
    });

    demos.splice(0, 0, '_default');

    let inner_html = demos.map(demo => `<option value="${demo}">${demo.slice(1)}</option>`)
    .join('\n');


    /** @type HTMLSelectElement */
    let select = document.getElementById('select');
    select.innerHTML = inner_html;

    app[demos[demos.length - 1]].run(1200, 1200 * 6 / 8);
    let old_val = select.value = demos[demos.length - 1];
  
    const load = async (demo) => {
      app[old_val].stop();
      app[demo].run(1200, 1200 * 6 / 8);
      old_val = demo;  
    }

    select.onchange = (e) => {
      if (old_val !== e.value) {
        load(select.value);
      }
    }
  })
  .catch(console.error)
