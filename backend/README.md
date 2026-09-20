# Stellerpool anchor (backend)

Stellerpool'un kendi işlettiği minimal **SEP-1 / SEP-10 / SEP-24** anchor sunucusu.
`TRYT` adında, **Türk lirasını temsil eden bir test varlığı** ihraç eder ve kullanıcının
cüzdanına gerçek bir on-chain ödemeyle gönderir.

## ⚠️ Bu ne DEĞİL

- **Gerçek banka veya ödeme kuruluşu entegrasyonu yok.** "TRY yatırdım" onayı, bir bankayı
  değil, bu sunucunun kendi (test amaçlı) onayını tetikler.
- **`TRYT` gerçek Türk lirası değildir.** Testnet'te var olan, 1:1 gerçek TL karşılığı
  bulunmayan bir demo varlığıdır. `stellar.toml`, form sayfası ve her yanıtta bu açıkça
  yazılıdır — bkz. `docs/altin-gunu-legal-boundary.md`.
- Üretim (mainnet) için düşünülmemiştir; kalıcı veritabanı, gerçek KYC, gerçek banka
  webhook'u yoktur (bkz. "Bilinçli basitleştirmeler").

Amaç: hackathon el kitabının "Anchor / Local Payments" gereksinimini — gerçek SEP
protokol akışıyla, ana ürün akışına gömülü şekilde — karşılamak, gerçek bir TRY rayı
kurmadan. Gerçek bir sağlayıcıya geçiş, tek bir alan adı değişikliği kadar basit olacak
şekilde tasarlandı (bkz. "Gerçek bir anchor'a geçiş").

## Kurulum

```bash
cd backend
npm install
npm run setup-issuer   # Testnet'te issuer + distribution hesabı üretir, TRYT ihraç eder
```

`setup-issuer` çıktısındaki `ISSUER_SECRET`, `DISTRIBUTION_SECRET`, `ASSET_CODE`
satırlarını `backend/.env` dosyasına yapıştırın (önce `cp .env.example .env`). Ayrıca
`JWT_SECRET`'ı rastgele bir dizeyle doldurun.

```bash
npm run dev     # tsx watch — geliştirme
npm run start   # tek seferlik çalıştırma
npm run build   # dist/ üretir (üretim benzeri çalıştırma için)
```

### `PUBLIC_BASE_URL` — https gereksinimi

SEP-10/24 istemcileri (ve `frontend/src/lib/anchor.ts`) interaktif pencere ve web-auth
adreslerinin **https** olmasını şart koşar. Yerelde geliştirirken bir tünel açın
(`cloudflared tunnel --url http://localhost:3001` — hesap gerektirmez, tek komut; veya
`ngrok http 3001`) ve `PUBLIC_BASE_URL`'i o https adresiyle güncelleyip sunucuyu yeniden
başlatın — `stellar.toml` ve tüm uç noktalar bu değeri baz alır. Bu, yalnızca test/demo
içindir: tünel geliştiricinin makinesi kapanınca ölür, adres her seferinde değişir.
**Kalıcı bir sunucuya deploy için → [`DEPLOY.md`](./DEPLOY.md).**

## Frontend'i buraya bağlamak

`frontend/.env` içinde:

```bash
VITE_ANCHOR_HOME_DOMAIN=<PUBLIC_BASE_URL'in host'u, örn. abc123.ngrok-free.app>
```

Havuz akışının da bu varlığı kullanması için (anchor'ı "core feature" yapan asıl adım):

```bash
VITE_POOL_ASSET_CODE=TRYT
VITE_POOL_ASSET_ISSUER=<setup-issuer çıktısındaki issuer public key>
```

Bundan sonra yeni oluşturulan havuzlar `TRYT`'yi kullanır; kullanıcı önce bu anchor'dan
TRYT yatırır, sonra o bakiyeyle havuza katılır — SEP akışı artık bir vitrin değil, gerçek
para giriş noktasıdır.

## Uç noktalar

| Uç nokta | SEP | Açıklama |
|---|---|---|
| `GET /.well-known/stellar.toml` | 1 | `WEB_AUTH_ENDPOINT`, `TRANSFER_SERVER_SEP0024`, `SIGNING_KEY`, `CURRENCIES` |
| `GET /auth?account=&home_domain=` | 10 | Challenge işlemi üretir |
| `POST /auth` `{transaction}` | 10 | İmzayı doğrular, JWT döner |
| `GET /sep24/info` | 24 | Desteklenen varlık/limitler |
| `POST /sep24/transactions/deposit/interactive` | 24 | İşlem açar, interaktif form adresini döner |
| `GET /sep24/interactive/:id` | 24 (özel) | Tutar formu (HTML) |
| `POST /sep24/interactive/:id/confirm` | 24 (özel) | "TRY yatırdım" onayı → on-chain ödeme dener |
| `GET /sep24/transaction?id=` | 24 | Durum sorgusu; `pending_trust` ise ödemeyi burada sessizce yeniden dener |
| `GET /health` | — | Sağlık kontrolü |

## Akış

1. Kullanıcı cüzdanıyla SEP-10 ile giriş yapar (`GET /auth` → imzala → `POST /auth` → JWT).
2. `POST /sep24/transactions/deposit/interactive` ile bir işlem açılır, `interactive` sayfa
   adresi döner (frontend bunu bir popup'ta açar).
3. Kullanıcı tutarı girip "TRY yatırdım, onayla (test)" der.
4. Sunucu **gerçek bir Stellar ödemesi** gönderir (dağıtım hesabından kullanıcıya, TRYT).
   Kullanıcının hesabında henüz trustline yoksa durum `pending_trust` olur.
5. Frontend `GET /sep24/transaction?id=` ile durumu poll'lar; kullanıcı trustline açtıktan
   sonraki bir sorguda sunucu ödemeyi kendiliğinden yeniden dener ve `completed` döner.

Uçtan uca gerçek bir doğrulama: `scripts/test-flow.mjs` (SEP-10 login → interactive deposit
→ onay → `pending_trust` → trustline aç → otomatik tamamlanma → bakiye kontrolü), gerçek
Testnet işlemleriyle çalıştırılıp doğrulandı.

## Bilinçli basitleştirmeler (hackathon kapsamı)

- **Bellek içi işlem deposu** (`src/lib/store.ts`): sunucu yeniden başlayınca kaybolur.
  Gerçek bir anchor kalıcı bir veritabanı kullanır.
- **"TRY yatırdım" onayı gerçek bir banka teyidi değil** — kullanıcının kendi butonuna
  bastığı an ödeme tetiklenir. Gerçek bir anchor bunu bir banka/ödeme kuruluşu webhook'una
  bağlar.
- **KYC yok** (SEP-12 uygulanmadı). Gerçek bir TRY anchor'ı yasal olarak KYC gerektirir.
- **Tek imzalı hesaplar varsayılır** (SEP-10 doğrulaması `verifyChallengeTxSigners` ile,
  eşik/çoklu imza senaryosu ele alınmadı) — hackathon demosu için yeterli.
- **Yalnızca deposit yönü var**, withdraw yok.

## Gerçek bir anchor'a geçiş

Gerçek, lisanslı bir TRY anchor sağlayıcısı bulunduğunda tek yapılması gereken
`frontend/.env`'de `VITE_ANCHOR_HOME_DOMAIN`'i o sağlayıcının alan adına çevirmek —
`frontend/src/lib/anchor.ts` zaten SEP-1'den okuduğu uç noktalara konuşacak şekilde
yazıldı, bu backend'e hiç bağımlı değil. Bu sunucu o zaman tamamen devre dışı bırakılabilir.
