# Teslim öncesi görev listesi — Pro Hackathon 2026

**Kaynak:** bu belge, projenin Pro Hackathon 2026 el kitabına karşı yapılan uyumluluk
analizinin (bkz. konuşma geçmişi / `docs/IMPLEMENTATION_LOG.md`) doğrudan sonucu.
Amaç: kalan işi **frontend / backend / kontrat** arasında net, çakışmasız, kabul
ölçütlü görevlere bölmek.

## ⚠️ Zaman uyarısı

Teslim son tarihi **20 Eylül 2026, 12:00**. Bu belgenin yazıldığı an "bugün" zaten
20 Eylül 2026 — yani kalan süre saatler mertebesinde olabilir, gün değil. Aşağıdaki
her bölüm **P0 (teslim için zorunlu, saatler içinde yapılabilir)**,
**P1 (zaman kalırsa)**, **P2 (hackathon sonrası yol haritası, şimdi yapma)** olarak
etiketlenmiştir. Gerçek kalan süre netleşince P0 listesi bile sığmıyorsa, sıradan
düşün: önce P0'ın en üstündeki maddeler, geri kalanı pitch'te "sıradaki adım" olarak
anlatılır — jüri kriteri 5 (Traction & Continuity) zaten "credible roadmap" arıyor,
her şeyin bitmiş olmasını değil.

## 0. Durum özeti (neden bu görevler)

- **Kontrat**: v10 (kura modu + 30 üye) Testnet'te yayında, 27 test yeşil, canlı
  senaryo kanıtlı. Bu taraf en sağlam kısım — bkz. madde 3.
- **En büyük boşluk — Ecosystem Fit / Anchor**: gerçek bir TRY anchor'ı yok, yalnızca
  SDF'in test anchor'ı (test varlığı üretir). El kitabı bunun **en ağırlıklı alt
  kriter** olduğunu açıkça yazıyor. Anchor ayrıca ana havuz akışına hiç bağlı değil
  (ayrı bir vitrin bileşeni) — "Core Feature" kriteri de bu yüzden karşılanmıyor.
- **User Experience değerlendirilemiyor**: yayınlanmış bir demo URL'i yok, cüzdanla
  uçtan uca hiç denenmemiş.
- **Presentation**: resmi sunum şablonu henüz doldurulmamış.
- **README**: kontrat ID'si eski (v9), kura/30-üye satırı artık yanlış (v10'da
  destekleniyor ama README hâlâ "desteklenmiyor" diyor).

## 1. Kontrat ekibi — kalan iş azalan öncelikte

Kontrat tarafı fonksiyonel olarak tamam; buradaki iş artık **diğer ekiplere
destek** ve küçük doğrulamalar.

**P0**
- [ ] Backend'in ihraç edeceği TRY-temsili test varlığının (bkz. madde 2) kontratla
  uyumunu doğrula: `create_pool`'a `token` olarak bu yeni SAC adresi verildiğinde
  `deposit`/`execute_round` sorunsuz çalışıyor mu — tek bir CLI denemesi yeterli
  (kontrat zaten herhangi bir SAC ile çalışacak şekilde asset-agnostik yazıldı, bu
  sadece bir doğrulama, kod değişikliği beklenmiyor).
- [ ] Frontend'in gerçek cüzdanla uçtan uca denemesi sırasında (madde 3) ortaya
  çıkabilecek kontrat-kaynaklı hataları hızlı triyaj et (varsa).

**P1**
- Yok — kapsam genişletmeye gerek yok, kalan saatler diğer eksik alanlara gitmeli.

**P2 (hackathon sonrası)**
- `docs/CONTRACT_HANDOFF.md`'nin "Bilinen sınırlar" bölümünde zaten yazılı: gerçek
  rastgelelik (commit-reveal), erken alıcı temerrüdü riski — bunlar bilinçli kapsam
  dışı, şimdi dokunma.

## 2. Backend ekibi — sıfırdan kurulmalı, en kritik iş

Şu an proje backend'siz. "Anchor" dediğimiz şey frontend'in doğrudan
`testanchor.stellar.org`'a konuştuğu bir istemci — kendi sunucumuz yok. El kitabının
en ağırlıklı kriterini karşılamak için **kendi minimal SEP-1/10/24 anchor
servisimizi** kurmamız gerekiyor. Bu, gerçek bir banka entegrasyonu **değil** — SEP
protokolüne tam uyumlu ama "TRY yatırıldı" onayının simüle edildiği (admin onayı /
sabit gecikme) bir servis. Dürüstlük şart: hem kodda hem pitch'te "kendi işlettiğimiz
test anchor'ı, gerçek banka rayı yok" net yazılmalı — aksi hem etik hem
(`docs/altin-gunu-legal-boundary.md`'de zaten işaretlenmiş) hukuki risk yaratır.

**P0 — minimum çalışan anchor**
- [ ] Kendi issuer hesabını oluştur (Testnet), TRY'yi temsil eden bir test varlığı
  ihraç et. Kod adı net "test/temsili" olduğunu belli etsin (örn. `TRYT` veya
  `TRYX`, "TRY" tek başına değil — gerçek TRY zannedilmesin).
- [ ] SEP-1 `stellar.toml` servis et: `CURRENCIES` (yeni varlık), `WEB_AUTH_ENDPOINT`,
  `TRANSFER_SERVER_SEP0024`, `SIGNING_KEY` alanları dolu olmalı.
- [ ] SEP-10 auth endpoint: challenge üret/doğrula, JWT dön. (`frontend/src/lib/anchor.ts`
  zaten generic bir SEP-10 istemcisi — sunucu tarafı eksik olan.)
- [ ] SEP-24 minimum uçlar: `GET /info`, `POST /transactions/deposit/interactive`,
  `GET /transaction?id=...`. Interactive deposit sayfası: kullanıcıdan tutar alır,
  "TRY yatırdım" onayı (gerçek ödeme değil, buton) sonrası arka planda kullanıcının
  Stellar hesabına gerçek bir on-chain transferle `TRYT` gönderir.
- [ ] `.env`/`frontend/.env`'deki `ANCHOR_HOME_DOMAIN` / `VITE_ANCHOR_HOME_DOMAIN`'i
  bu yeni sunucunun domainine (veya localhost/tünel adresine, demo sırasında) çevir.
- [ ] **Kabul ölçütü**: frontend'ten gerçek bir cüzdanla SEP-10 login + SEP-24
  interactive deposit tam döngüsü çalışıyor, kullanıcı bakiyesinde `TRYT` görünüyor.

**P1 — zaman kalırsa**
- [ ] SEP-12 KYC-lite form (ad, e-posta — gerçek doğrulama yok, sadece akış tamlığı
  için).
- [ ] Withdraw yönü (kullanıcı `TRYT`'yi geri "TRY"ye çevirme simülasyonu).
- [ ] Deposit sonrası havuza otomatik "bakiye yükle" köprüsü (aşağıda frontend
  madde 3, P0'daki manuel bağlantının otomasyonu).

**P2 (hackathon sonrası, pitch'te yol haritası olarak anlat)**
- Gerçek bir lisanslı TRY anchor sağlayıcısıyla görüşme (SEP-24 uyumlu bir üçüncü
  taraf), gerçek banka/ödeme kuruluşu entegrasyonu, KYC/AML.

## 3. Frontend ekibi

**P0**
- [ ] **Anchor'ı ana akışa bağla**: en azından "bakiye yükle" / "TRY ile katıl" gibi
  belirgin bir adım olarak `CreateView.vue` veya `PoolView.vue`'ya ekle — şu an
  `AnchorDemo.vue` yalnızca `HomeView`/`ShowcaseView`'de, havuz akışıyla hiç teması
  yok. Tam otomasyon şart değil; en azından kullanıcı "önce TRY yatır, sonra havuza
  katıl" akışını gözle takip edebilmeli.
- [ ] Backend hazır olunca (madde 2) `VITE_ANCHOR_HOME_DOMAIN`'i o sunucuya çevir,
  gerçek bir cüzdanla SEP-10 + SEP-24 interactive deposit'i dene.
- [ ] **Gerçek cüzdanla (Freighter) uçtan uca tam döngü**: havuz oluştur → üye
  katıl → şartları öner/onayla → başlat → öde → (kura modundaysa) kura çek →
  execute_round → tamamlandı. Ekran görüntüsü veya kısa video al (pitch/README için
  kanıt).
- [ ] **Frontend'i public bir URL'e deploy et** (Vercel/Netlify vb.), build'de
  güncel `.env` değerlerini kullan (özellikle yeni v10 kontrat ID'si ve
  `VITE_MAX_MEMBERS=30` — bunlar zaten `frontend/.env`'de hazır, sadece deploy
  ortamına taşınmalı).
- [ ] **README güncelle** (kontrat ekibi buna dokunmuyor, bu görev frontend/dokümantasyon
  tarafında):
  - Kontrat ID'sini v9'dan v10'a (`CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB`)
    güncelle, WASM hash ve tx linklerini `docs/IMPLEMENTATION_LOG.md` "Phase 13"
    bölümünden al.
  - "Kura modu ve 30 üye: 🟡 kontrat henüz desteklemiyor" satırını "✅ Testnet'te"
    yap.
  - Demo URL'i ekle (yukarıdaki deploy adımı bitince).
  - Backend/anchor domaini gerçek servise geçtiyse o satırı da güncelle.
- [ ] **Resmi Stellar sunum şablonunu kopyala ve doldur** (kopyasını al, orijinali
  düzenleme). Zorunlu teslim maddesi, henüz yapılmadı.

**P1 — zaman kalırsa**
- [ ] Kura/30 üye UI'ını canlı v10 kontratına karşı manuel dene (arayüz zaten zincirdeki
  kontrat yeteneğini okuyup kura seçeneğini otomatik açıyor — tek eksik gerçek bir
  denemenin yapılmış olması).
- [ ] `docs/architecture/architecture.md`'deki Anchor akış diyagramını, gerçek backend
  servisini yansıtacak şekilde güncelle (şu an SDF test anchor'ını varsayıyor).

**P2**
- Passkey/smart wallet desteği — el kitabı bunu açıkça "bonus, zorunlu değil" diyor,
  şimdi uğraşma.

## 4. Teslim portalı kontrol listesi (herkes, son adım)

- [ ] Takım adı + tüm üyelerin isim/iletişim bilgisi
- [ ] GitHub repo linki (zaten açık: `github.com/yusufarslan44/Stellerpool`)
- [ ] Canlı demo URL'i (madde 3, P0)
- [ ] Sunum linki, "Anyone with the link can view" olarak paylaşım ayarı açık
- [ ] Track seçimi: **Genesis**
- [ ] Submission formunda kontrat ID'lerinin güncel (v10) olduğundan emin ol
