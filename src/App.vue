<template>
  <div>
    <nav class="navbar is-dark is-fixed-top" role="navigation" aria-label="main navigation">
      <div class="navbar-brand">
        <router-link class="navbar-item" to="/">
          <img :src="logo" alt="logo" />
        </router-link>
        <div class="navbar-item is-hidden-desktop">{{ now }}</div>
        <a
          role="button"
          class="navbar-burger"
          :class="{ 'is-active': isMenuActive }"
          aria-label="menu"
          aria-expanded="false"
          @click="isMenuActive = !isMenuActive"
        >
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
        </a>
      </div>
      <div class="navbar-menu" :class="{ 'is-active': isMenuActive }">
        <div class="navbar-start">
          <router-link class="navbar-item" to="/">Home</router-link>
          <router-link class="navbar-item" to="/settings">Settings</router-link>
        </div>
        <div class="navbar-end">
          <div class="navbar-item is-hidden-touch">{{ now }}</div>
        </div>
      </div>
    </nav>
    <section class="section">
      <router-view />
    </section>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import logoUrl from "./assets/logo.svg";

const logo = logoUrl;
const now = ref("");
const isMenuActive = ref(false);
let timer;

onMounted(() => {
  now.value = new Date().toLocaleTimeString();
  timer = setInterval(() => {
    now.value = new Date().toLocaleTimeString();
  }, 1000);
});

onUnmounted(() => {
  clearInterval(timer);
});
</script>

<style scoped>
.navbar-item img {
  max-height: 3rem;
}
</style>
