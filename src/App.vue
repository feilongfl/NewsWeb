<template>
  <div>
    <nav>
      <router-link to="/">Home</router-link> |
      <router-link to="/settings">Settings</router-link>
    </nav>
    <router-view />
  </div>
</template>

<script setup></script>

<style scoped>
nav {
  padding: 1rem;
  background: #f0f0f0;
}
nav a {
  margin-right: 1rem;
}
</style>
