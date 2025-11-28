<script lang="ts">
  interface Props {
    title: string;
  }

  let { title }: Props = $props();
</script>

<header class="header">
  <h1>{title}</h1>
</header>

<style>
  .header {
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #2a2a4a;
    background: #16162a;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: #fff;
  }
</style>
