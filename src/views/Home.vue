<template>
  <div class="container">
    <div v-for="item in parsedItems" :key="item.link" class="card mb-4">
const parsedItems = computed(() => visibleItems.value);
  const tx = db.transaction("news");
  all.sort((a, b) => (b.ts || 0) - (a.ts || 0));
  items.value = all.slice(0, 10);
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;
        let news;
          news = JSON.parse(text);
          continue;
        news.ts = Date.now();
        txw.store.put(news);
  for (const item of latest) {
    if (item.text) {
      let text = item.text;
      if (keyBytes) {
        try {
          const ciphertext = CryptoJS.enc.Base64.parse(item.text);
          text = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
            mode: CryptoJS.mode.ECB,
            padding: CryptoJS.pad.Pkcs7,
          }).toString(CryptoJS.enc.Utf8);
        } catch {
          // failed to decrypt, keep original text
        }
      }
      try {
        const data = JSON.parse(text);
        Object.assign(item, data);
      } catch {
        item.raw = text;
      }
      delete item.text;
      tx.store.put(item);
    }
  }
  items.value = latest;
  await tx.done;
}

async function connectStream() {
  const db = await dbPromise;
  const pwd = (await db.get("settings", "password")) || "";
  const url =
    (await db.get("settings", "streamUrl")) ||
    "https://feilongfl-1.gl.srv.us/stream";
  const delay = (await db.get("settings", "reconnectDelay")) || 5000;
  const keyBytes = pwd
    ? CryptoJS.enc.Utf8.parse(pwd.padEnd(8, "\0").slice(0, 8))
    : null;

  let reader;
  try {
    const resp = await fetch(url);
    reader = resp.body.getReader();
  } catch (e) {
    console.error("stream connect error", e);
    setTimeout(connectStream, delay);
    return;
  }

  const decoder = new TextDecoder();
  let buf = "";
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) throw new Error("stream closed");
      buf += decoder.decode(value, { stream: true });
      let lines = buf.split(/\r?\n/);
      buf = lines.pop();
      for (let line of lines) {
        if (!line.startsWith("data:")) continue;
        let payload = line.slice(5).trim();
        if (!payload) continue;
        let text = payload;
        if (keyBytes) {
          try {
            let ciphertext = CryptoJS.enc.Base64.parse(payload);
            text = CryptoJS.DES.decrypt({ ciphertext }, keyBytes, {
              mode: CryptoJS.mode.ECB,
              padding: CryptoJS.pad.Pkcs7,
            }).toString(CryptoJS.enc.Utf8);
            if (!text) continue;
          } catch {
            continue;
          }
        }
        let news = { id: Date.now() + Math.random() };
        try {
          Object.assign(news, JSON.parse(text));
        } catch {
          news.raw = text;
        }
        const txw = db.transaction("news", "readwrite");
        txw.store.add(news);
        items.value.unshift(news);
        if (items.value.length > 10) items.value.pop();
      }
    }
  } catch (e) {
    console.error("stream read error", e);
  }
  setTimeout(connectStream, delay);
}

onMounted(async () => {
  await loadItems();
  connectStream();
});
</script>
