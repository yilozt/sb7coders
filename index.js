// For more comments about what's going on here, check out the `hello_world`
// example.
import('./pkg')
  .then(app => app["hello_world"]())
  .catch(console.error)
