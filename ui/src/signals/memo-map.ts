
// Subset of ReadonlyMap, with only the get function
export interface GetonlyMap<K, V> {
  get(key: K): V;
}

export class MemoMap<K, V> implements GetonlyMap<K, V> {
  map = new Map<K, V>();

  constructor(protected newValue: (hash: K) => V) {}

  get(hash: K): V {
    if (!this.map.has(hash)) {
      this.map.set(hash, this.newValue(hash));
    }
    return this.map.get(hash)!;
  }
}

