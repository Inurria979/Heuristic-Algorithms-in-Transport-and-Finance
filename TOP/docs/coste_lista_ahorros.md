# ¿Merece la pena compartir la lista de ahorros entre ejecuciones de GRASP?

Medición del 11-10-2026. Responde a la optimización pendiente 6 de
[`scratchpad.md`](scratchpad.md): **no merece la pena**. El coste que se ahorraría es solo
el 3,5–5,4 % de cada ejecución de GRASP.

---

## 1. La pregunta

### Cómo funciona ahora la rejilla

`pjs grid` reparte el trabajo en tareas independientes. Para cada pareja (instancia, α) hay
21 tareas (`alpha_grid.rs`, `run_task`):

| Tarea | Qué hace |
|---|---|
| t = 0 | Construcción determinista, con `pjs(inst, alpha)` |
| t = 1…10 | 10 ejecuciones de GRASP geométrico (β = 0.3), cada una de 10 iteraciones |
| t = 11…20 | 10 ejecuciones de GRASP softmax (τ = 1), cada una de 10 iteraciones |

Cada ejecución de GRASP (`grasp.rs`, `grasp()`) empieza siempre igual:

```rust
let list = SavingsList::new(inst, cfg.alpha);                        // (1) lista de ahorros
let (mut best, _) = construct(inst, &list, &mut Greedy::default());  // (2) solución determinista
// (3) después, 10 construcciones aleatorias con la misma lista
```

### El trabajo repetido

Los pasos (1) y (2) **solo dependen de la instancia y de α**, no de la semilla ni de la
aleatorización. Aun así, se calculan en cada una de las 21 tareas de la pareja:

| | Ahora | Si se compartieran |
|---|---|---|
| Listas de ahorros en toda la rejilla | 170 667 | 8 127 (387 × 21) |
| Construcciones deterministas | 170 667 | 8 127 |

### La optimización propuesta

La idea era construir la lista y la solución determinista una vez por (instancia, α) y
pasárselas por referencia a las 20 ejecuciones.

- **Los resultados no cambiarían:** la lista es determinista y cada ejecución conserva su
  semilla.
- **El código sí se complicaría:**
  - `grasp()` tendría que recibir la lista y la solución inicial;
  - la rejilla pasaría de una lista plana de tareas a una tarea por pareja con paralelismo
    anidado.

Antes de complicar el código, había que saber **qué parte del tiempo de una ejecución se va
en (1) + (2)**. Si es poca, no compensa.

---

## 2. Cómo se midió

### Instancias

Se midieron dos conjuntos, el de menos nodos y el de más:

| Conjunto | Nodos | Instancias medidas | Comentario |
|---|---|---|---|
| **p2** | 21 | 33 | Todas las de p2 son factibles |
| **p7** | 102 | 58 | Se excluyen las 2 en las que ningún cliente es alcanzable |

En cada instancia se usaron **los 21 valores de α** de la rejilla (0, 0.05, …, 1), igual
que en el experimento real.

### Qué se cronometró

Para cada (instancia, α), y con la misma configuración que la rejilla:

| Medida | Código | Qué incluye |
|---|---|---|
| Lista de ahorros | `SavingsList::new(inst, alpha)` | Generar los ~n² arcos con su ahorro enriquecido y ordenarlos |
| Construcción determinista | `construct(inst, &list, &mut Greedy)` | Solución dummy + recorrer la lista fusionando + seleccionar m rutas |
| GRASP geométrico | `grasp()` con β = 0.3 y 10 iteraciones | **La ejecución completa:** (1) + (2) + 10 construcciones aleatorias |
| GRASP softmax | `grasp()` con τ = 1 y 10 iteraciones | Lo mismo con el softmax |

La **parte del coste fijo** es (lista + determinista) / ejecución completa de GRASP. Es
justo lo que la optimización se ahorraría en cada ejecución.

### Condiciones

- **Equipo:** AMD Ryzen 5 5500U (6 núcleos, 12 hilos).
- **Un solo hilo:** sin rayon, para medir el coste de cada operación sin interferencias.
- **Compilación:** el mismo perfil `release` del proyecto (fat LTO, `codegen-units = 1`,
  `panic = "abort"`, `target-cpu=native`).
- **`std::hint::black_box`** sobre cada resultado, para que el compilador no elimine el
  cálculo.
- **Semillas:** una distinta en cada repetición de GRASP, para no medir siempre la misma
  secuencia aleatoria.
- **Sin tocar el proyecto:** se midió con una copia del crate en el scratchpad de la sesión,
  con un subcomando `bench` añadido. El código está en el apéndice.
- **Repeticiones:**
  - **p2:** 20 por medida (y otra ronda con 1 para comparar).
  - **p7:** solo 1. Con 20, la medición iba a tardar decenas de minutos y se detuvo.

---

## 3. Resultados

Tiempos medios por llamada, promediados sobre todas las (instancia, α) del conjunto:

| | p2 (21 nodos), 20 rep. | p2 (21 nodos), 1 rep. | p7 (102 nodos), 1 rep. |
|---|---|---|---|
| Lista de ahorros | 3,1 µs | 5,0 µs | 433 µs |
| Construcción determinista | 3,9 µs | 5,6 µs | 135 µs |
| **Coste fijo (las dos)** | **7,0 µs** | **10,6 µs** | **568 µs** |
| Ejecución de GRASP geométrico (10 it.) | 228 µs | 234 µs | 13,4 ms |
| Ejecución de GRASP softmax (10 it.) | 188 µs | 198 µs | 16,1 ms |
| **Parte del coste fijo, geométrico** | **3,1 %** | **4,5 %** | **4,2 %** |
| **Parte del coste fijo, softmax** | **3,7 %** | **5,4 %** | **3,5 %** |

Salida original:

```
set p2: 33 instances x 21 alphas, 21 nodes, 20 reps each (single thread)
  savings list             3.12 us
  greedy construction      3.85 us
  fixed cost (both)        6.97 us
  GRASP geometric 10     228.32 us   fixed share   3.1 %
  GRASP softmax   10     188.10 us   fixed share   3.7 %

set p2: 33 instances x 21 alphas, 21 nodes, 1 reps each (single thread)
  savings list             4.98 us
  greedy construction      5.62 us
  fixed cost (both)       10.60 us
  GRASP geometric 10     233.56 us   fixed share   4.5 %
  GRASP softmax   10     197.60 us   fixed share   5.4 %
set p7: 58 instances x 21 alphas, 102 nodes, 1 reps each (single thread)
  savings list           432.80 us
  greedy construction    134.82 us
  fixed cost (both)      567.62 us
  GRASP geometric 10   13418.27 us   fixed share   4.2 %
  GRASP softmax   10   16094.94 us   fixed share   3.5 %
```

### Sobre la precisión

- **En p2 los tiempos son de microsegundos, y el ruido pesa.** Con 1 repetición, el coste
  fijo sale un 50 % mayor que con 20 (10,6 frente a 7,0 µs). La primera llamada a cada
  función paga fallos de caché y de predicción que se amortizan al repetir.
- **En p7 los tiempos son de cientos de µs a milisegundos**, así que una sola repetición ya
  es razonablemente estable.
- **La conclusión es la misma en las tres mediciones:** el coste fijo está entre el 3 y el
  5,5 %.

---

## 4. Interpretación

### Cuánto se ahorraría

- En cada ejecución de GRASP se ahorraría un ~4 % de su tiempo.
- Las ejecuciones de GRASP son 20 de las 21 tareas de cada pareja, y la tarea determinista
  seguiría haciéndose. El ahorro total sería, por tanto, **≈ 4 % de la rejilla: unos 6 s de
  los 140 s**.
- A cambio habría que cambiar la firma de `grasp()`, reestructurar `solve_all` con
  paralelismo anidado y volver a comprobar los CSV byte a byte.

**Decisión: descartada.** No compensa la complicación del código.

### Por qué el coste fijo es tan pequeño

El scratchpad suponía que, con solo 10 iteraciones, la lista pesaría mucho. La suposición
era razonable, pero no se cumple porque **una construcción aleatoria es mucho más cara que
la determinista**:

| | p2 | p7 |
|---|---|---|
| Construcción determinista | ~4–6 µs | 135 µs |
| Construcción aleatoria, geométrica | ~22 µs | ~1,3 ms |
| Construcción aleatoria, softmax | ~18 µs | ~1,6 ms |
| Aleatoria / determinista | ~4–5× | ~10–12× |

(Las construcciones aleatorias salen de restar el coste fijo a la ejecución completa y
dividir entre 10.)

Las 10 construcciones aleatorias pesan unas 20 veces más que la lista y la determinista
juntas, y eso diluye el coste fijo.

### Por qué la construcción aleatoria es más lenta (hipótesis, sin verificar)

Las dos recorren la lista entera (`for _ in 0..list.arcs.len()` en `construct`) e intentan
una fusión por arco. Hay tres posibles motivos de la diferencia:

1. **El coste del selector.** `Greedy` solo devuelve el siguiente índice. El geométrico
   calcula un logaritmo y un número aleatorio por arco y quita un elemento de un `Vec`. El
   softmax actualiza un árbol de sumas en O(log n) y calcula exponenciales. Con ~10 000
   arcos en p7, eso son decenas de miles de operaciones caras por construcción.
2. **El acceso a memoria.** El voraz recorre la lista en orden. El aleatorio salta entre
   posiciones, lo que aprovecha peor la caché.
3. **Otra secuencia de fusiones.** El orden aleatorio produce otras rutas intermedias, y
   puede que `try_merge` pase más a menudo del filtro rápido al cálculo exacto.

Nada de esto se ha medido. Si algún día hace falta acelerar la rejilla, **el sitio donde
mirar son los selectores aleatorios, no la lista de ahorros**.

### Qué cambia con la búsqueda local

La conclusión se refuerza. Con búsqueda local en cada iteración, cada iteración costará más,
y el coste fijo pesará todavía menos que el 4 % actual.

---

## Apéndice: código de la medición

Subcomando temporal añadido a una copia del crate. No está en el proyecto. Para repetir la
medición:

1. Copiar `TOP/src/rust` a una carpeta temporal.
2. Añadir este fichero como `src/bench.rs` y `mod bench;` en `main.rs`.
3. Al principio de `main()`, después de leer el comando, añadir:

   ```rust
   if command == "bench" {
       let insts = io::load_all(Path::new(&args[0])).unwrap();
       for set in ["p2", "p7"] { bench::bench(&insts, set, args[1].parse().unwrap()); }
       return;
   }
   ```

4. Compilar con `cargo build --release` y ejecutar desde `TOP/`:
   `pjs.exe bench src/data/top-data <repeticiones>`.

```rust
//! Temporary timing of the fixed cost of a GRASP run (savings list + greedy construction).
use crate::alpha_grid::{grid_alpha, ALPHAS};
use crate::grasp::{grasp, GraspConfig, Randomization};
use crate::heuristic::{construct, Greedy, SavingsList};
use crate::model::Instance;
use std::hint::black_box;
use std::time::Instant;

fn time_us<F: FnMut()>(reps: usize, mut f: F) -> f64 {
    let start = Instant::now();
    for _ in 0..reps {
        f();
    }
    start.elapsed().as_secs_f64() * 1e6 / reps as f64
}

pub fn bench(instances: &[Instance], set: &str, reps: usize) {
    let insts: Vec<&Instance> = instances.iter().filter(|i| i.set() == set && i.reachable_count() > 0).collect();
    let (mut list_t, mut greedy_t, mut geo_t, mut soft_t, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for inst in &insts {
        for k in 0..ALPHAS {
            let alpha = grid_alpha(k);
            list_t += time_us(reps, || { black_box(SavingsList::new(inst, alpha)); });
            let list = SavingsList::new(inst, alpha);
            greedy_t += time_us(reps, || { black_box(construct(inst, &list, &mut Greedy::default())); });
            for (r, acc) in [(Randomization::Geometric { beta: 0.3 }, &mut geo_t), (Randomization::Softmax { tau: 1.0 }, &mut soft_t)] {
                let cfg = GraspConfig { alpha, randomization: r, iterations: 10, time_limit: None };
                let mut seed = 0;
                *acc += time_us(reps, || { seed += 1; black_box(grasp(inst, &cfg, seed)); });
            }
            n += 1.0;
        }
    }
    let (l, g, geo, soft) = (list_t / n, greedy_t / n, geo_t / n, soft_t / n);
    println!("set {set}: {} instances x {ALPHAS} alphas, {} nodes, {reps} reps each (single thread)", insts.len(), insts[0].n());
    println!("  savings list        {l:9.2} us");
    println!("  greedy construction {g:9.2} us");
    println!("  fixed cost (both)   {:9.2} us", l + g);
    println!("  GRASP geometric 10  {geo:9.2} us   fixed share {:5.1} %", 100.0 * (l + g) / geo);
    println!("  GRASP softmax   10  {soft:9.2} us   fixed share {:5.1} %", 100.0 * (l + g) / soft);
}
```
