# Finger Twister

Mobilna hra v Ruste s Macroquad 0.4.13. Pouziva rovnaky zaklad a Android Docker build ako susedny projekt RicochetRobots.

## Pravidla

- Vyber samostatne pocet stlpcov (sirka) a riadkov (vyska), kazdy od 3 do 6, a limit 3, 5 alebo 8 sekund na novy prst.
- Po stlaceni HRAT uvolni displej. Potom zacne prvy casovy limit.
- Pocas hry zabera mriezka celu dostupnu plochu obrazovky, bez titulku, nastaveni a tlacidiel. Bunky su kruhy rozmiestnene podla sirky aj vysky zariadenia. Dotyk musi zostat v kruhu; medzery medzi kruhmi nepatria do ziadnej bunky.
- Poloz jeden novy prst na volne zvyraznene policko. Pozadovana farba ma pulzujuci biely ramik; pasik priamo v jej polickach ukazuje zostavajuci cas. Rimske cisla I–IV rozlisuju farby aj bez vnimania odtiena.
- Kazdy predchadzajuci prst musi zostat na svojom povodnom policku. Pohyb v ramci policka je povoleny.
- Zdvihnutie prsta, posun mimo jeho policka, nespravna farba, obsadene policko, viac novych prstov naraz alebo vyprsanie casu znamena prehru.
- Kazdy spravny prst obnovi casovy limit. Dalsia farba ma vzdy aspon jedno volne policko. Zaplnenie celej mriezky znamena vyhru.

Pocet sucasnych dotykov obmedzuje telefon. Ked uz nevie zaznamenat dalsi prst, hra skonci vyprsanim casu. Hra sleduje skutocne ID dotykov; na mobile sa prsty nezamykaju softverovo.

## Spustenie na pocitaci

```powershell
cargo run
```

Pre skusanie viacerych prstov mysou zapni v menu **MYS: ZAPNUT TRENING**. Kliknutie ponecha virtualny prst na policku, opatovne kliknutie ho zdvihne a sposobi prehru. Ide o trening, nie o fyzicke drzanie dotykov. Bez treningu mys predstavuje jeden drzany prst. Dotykova obrazovka pouziva skutocny multitouch.

## Android APK

Spusti Docker Desktop v rezime Linux containers a potom:

```powershell
powershell -ExecutionPolicy Bypass -File .\build-android.ps1
```

Vystup: `target/android-artifacts/release/apk/fingertwister.apk`.

Build pouziva ARM64, SDK/target API 36 a minimum API 21. Skripty overuju manifest, podpis a zhodu Java loadera s native kniznicou. Docker volumes uchovavaju cache aj vyvojovy podpisovy kluc. APK je podpisane vyvojovym klucom pre lokalne testovanie. iOS build nie je nakonfigurovany.

## Kontroly

```powershell
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo fmt --check
```

Testy overuju pridavanie prstov, zdvihnutie, posun, vymenu ID, zle a obsadene policko, viac novych dotykov, casovy limit, dostupnost zadanej farby aj uplne zaplnenie mriezky. Fyzicky multitouch treba overit na telefone.
