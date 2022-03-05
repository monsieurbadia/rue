<script lang="ts">
  import { defineComponent, onMounted } from 'vue'
  import init, { greet, render } from 'wasm';

  export default defineComponent({
    setup() {
      // run the wasm code
      const run = async () => {
        // first, load the wasm module
        await init();
        // then greet the world
        greet("world");
        // then, render the scene
        render();
      };

      // run the wasm code when the component is mounted
      onMounted(run);
    },
  })
</script>

<template>
  <canvas id="renderer" />
</template>

<style scoped>
  canvas {
    width: 400px;
    height: 400px;
    border: 1px solid black;
  }
</style>
