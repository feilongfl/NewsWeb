<template>
  <div>
    <nav class="navbar is-light" role="navigation" aria-label="main navigation">
      <div class="navbar-menu is-active">
        <div class="navbar-start">
          <router-link class="navbar-item" to="/">Home</router-link>
          <router-link class="navbar-item" to="/settings">Settings</router-link>
        </div>
      </div>
    </nav>
    <section class="section">
      <router-view />
    </section>
  </div>
</template>

<script setup></script>

<style scoped></style>
