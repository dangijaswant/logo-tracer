# Optical Corrections Every Logo Designer Should Know — Logo Geek

**Source:** https://logogeek.uk/logo-design/optical-corrections/
**Role in this project:** the *final post-processing* reference. After geometry is clean,
these are the perceptual adjustments that make a logo "look right" to the human eye even
when it deviates from mathematically-perfect coordinates. Applied last, sparingly, and
**configurable/optional** (they intentionally break the grid).

> Core thesis: "Use grids only as a guide, not a rule. **Correct by eye.**" Mathematically
> perfect can look *wrong*; slightly "imperfect" can look perfect.

---

## The four optical phenomena

### 1. Perfect can look imperfect (and vice versa)
- Applying "perfect" guides to an already-balanced mark can make it look *worse*
  (the Google "G" example: the math-corrected version looked unbalanced).
- → **Don't over-snap.** Our keyline/axis snapping must be *tolerance-bounded*: snap only
  near-misses, and make snapping strength configurable so we don't "correct" intentional
  optical balance into stiffness. (Tie-in: guide's acceptance criterion "intended rounded
  edges remain smooth" — don't flatten deliberate shapes.)

### 2. Overshoot (round/pointed shapes vs flat shapes)
- A circle the **same mathematical height** as a square **looks too small**. To look equal,
  round (and pointed) shapes must **extend slightly beyond** the flat shape's bounds — this
  extension is **overshoot**.
- Applies to round letters (O, C, S) and apexes (A, V) vs flat-topped letters (E, H, T).
- → **Optional optical pass:** detect round/pointed extrema that sit on a shared baseline/
  cap-line and allow a small configurable overshoot rather than hard-snapping them flush to
  the line. Default off / very small; must be a toggle (it deviates from the source pixels).

### 3. Gestalt (perceptual grouping)
- The brain organizes parts into "unified wholes" via similarity, continuation, **closure**,
  proximity, figure/ground (e.g. WWF panda = closure of separate shapes).
- "Design what's there **and** what's not there."
- → For us: respect **figure/ground** (the 2-color logo = dark figure on light ground;
  counters are ground). Preserve closure — don't fill in or break counters/negative space.
  Keep separate-but-related shapes consistent so they read as one mark (reinforces the
  keyline-consistency goal).

### 4. Irradiation phenomenon (light-on-dark looks fatter)
- The **same** mark looks **fatter** when light-on-dark vs dark-on-light (contrast illusion).
- Fix: for a white/knockout version, **slightly thin** the shape (stroke-expand-subtract by
  eye) so it looks optically equal to the dark-on-light version.
- → **Optional knockout adjustment:** if/when we emit an inverted color variant, offer a
  small configurable inward offset (negative outline offset) to compensate. Default off.

---

## How these constrain our pipeline (important guardrails)
- These are **perceptual, opinionated, and rule-breaking by design.** Therefore in the Rust
  logo-tracer they must be:
  - **applied last** (after clean geometry, lines, corners, fitting, primitives);
  - **off or minimal by default** (the tracer's job is faithful reconstruction first);
  - **configurable** (overshoot amount, knockout thinning) and **clearly documented**;
  - **never** allowed to distort by skew/rotation (consistent with Material's rule).
- They are the reason axis/keyline snapping must use **tolerances** and not force every edge
  to an exact grid — over-snapping destroys intentional optical balance.

---

## Distilled rules → Rust logo pipeline (final post-processing, optional)
1. **Tolerance-bounded snapping** only — never hard-correct intentional optical offsets.
2. **Overshoot (optional, default ~0):** allow round/pointed extrema a small extension past
   shared baselines/cap-lines instead of flush snapping.
3. **Preserve figure/ground & closure:** keep counters and negative space intact.
4. **Knockout thinning (optional, default off):** inward offset for inverted/white variants
   to counter irradiation.
5. All optical passes are **toggleable, documented, last in the pipeline**, and never
   skew/rotate geometry.
