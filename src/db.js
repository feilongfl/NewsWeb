import { openDB } from "idb";

export const dbPromise = openDB("news-db", 2, {
  upgrade(db) {
    if (db.objectStoreNames.contains("news")) {
      db.deleteObjectStore("news");
    }
    db.createObjectStore("news", { keyPath: "link" });
    if (!db.objectStoreNames.contains("settings")) {
      db.createObjectStore("settings");
    }
  },
});
