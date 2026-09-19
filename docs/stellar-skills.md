# Stellar Skills — Kullanım Notları

Kaynak: https://skills.stellar.org (19 Eylül 2026'da incelendi). Tüm skill metadata'sı: https://skills.stellar.org/llms.txt

Hackathon jüri kriteri (Ecosystem Fit): *"Makes effective use of Stellar SDKs, CLI, and Skills resources"* ve *"References the relevant Stellar Skills within the documentation."* Bu dosya hangi skill'i nerede kullandığımızı belgeler.

> Notlar skill sayfalarından özetlenmiştir. Kodlamadan önce ilgili `SKILL.md` dosyasını okuyun, sürümler ve API adları değişmiş olabilir.
> Topluluk skill'leri SDF tarafından denetlenmez. Kod veya komut çalıştırmadan önce içeriğini kontrol edin.

## Projede kullanılan skill'ler

| Skill | Adres | Nerede kullanıyoruz |
|---|---|---|
| Stellar Smart Contracts (resmi) | skills.stellar.org/skills/smart-contracts/SKILL.md | `contracts/`: Soroban kurulum, storage/TTL, auth, event, test, deploy |
| Frontend & Wallets (resmi) | skills.stellar.org/skills/dapp/SKILL.md | `frontend/`: SDK, Wallets Kit, kontrat çağrısı, event okuma |
| Stellar Assets & SAC (resmi) | skills.stellar.org/skills/assets/SKILL.md | Havuz/teminat token'ı, trustline, 7 ondalık |
| SEPs, CAPs & Ecosystem (resmi) | skills.stellar.org/skills/standards/SKILL.md | Anchor akışı (SEP-1/10/24/6/38) |
| RPC & Horizon APIs (resmi) | skills.stellar.org/skills/data/SKILL.md | Havuz durumu, event ve işlem sorgulama |
| Anchors (topluluk, CheesecakeLabs) | github.com/CheesecakeLabs/stellar-anchor-skill | Anchor entegrasyonu (istemci tarafı) |
| Soroban Common Mistakes (topluluk) | github.com/mariaelisaaraya/stellar-security-guide | Kontrat için 23 maddelik güvenlik kontrol listesi |
| Stellar Integration Finder, SCF Submission Radar, Stellar Scout (LumenLoop) | github.com/lumenloop/lumenloop-skills | Traction ve "sonraki adım" (SCF/InstaAwards) |

## Frontend (Frontend & Wallets skill'inden)

- Paketler: `@stellar/stellar-sdk`, `@stellar/freighter-api`, Stellar Wallets Kit (`@creit-tech/stellar-wallets-kit`, **JSR üzerinden**: `npx jsr add @creit-tech/stellar-wallets-kit`). Skill Node.js 22+ öneriyor.
- SDK v16+ ESM-first. `@stellar/stellar-base` ayrıca kuruluysa kaldırın, ikisi birlikte `instanceof` kontrollerini bozar.
- Wallets Kit v2: `StellarWalletsKit.init({ modules: defaultModules(), network: Networks.TESTNET })`, bağlanma `authModal()`, imza `signTransaction(xdr)`.
- Kontrat çağrısı: `contract.Client.from<T>({ contractId, rpcUrl, networkPassphrase, publicKey, signTransaction })`. Okuma çağrıları imza gerektirmez (simülasyon). Yazma: `tx.signAndSend()`.
- Tip üretimi: `stellar contract bindings typescript --wasm <yol>.wasm --output-dir src/types`.
- Event okuma: `rpc.getEvents({ filters: [{ contractIds: [...], topics: [["*"]] }], startLedger, limit })`. İşlem takibi: `rpc.pollTransaction(hash)`.
- Testnet: RPC `https://soroban-testnet.stellar.org`, Horizon `https://horizon-testnet.stellar.org`, passphrase `Networks.TESTNET`.
- İmzadan önce cüzdanın ağı ile uygulamanın ağının aynı olduğunu doğrulayın.

## Token, teminat ve trustline (Assets & SAC skill'inden)

- Stellar asset 7 ondalıklıdır: `1000000` = `0.1`. Arayüzde `10^7`'ye bölün, tutarları JS `number` yerine string/BigInt tutun.
- Kontrat (C-hesabı) SAC bakiyesini trustline olmadan tutar. **Kullanıcı hesabı (G-hesabı) trustline'sız asset alamaz, transfer başarısız olur.** Bu yüzden `join_pool` sırasında trustline kontrol edilmeli, arayüz de eksikse trustline eklemeyi önermeli.
- SAC adresi: `new Asset(code, issuer).contractId(Networks.TESTNET)`.
- Testnet USDC ihraççısı: `GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5` (Horizon testnet'te doğrulandı: yaklaşık 69 bin hesap, dolaşımda büyük arz, mainnet'te yok). **Dikkat:** skill sayfasında geçen `GA5ZSEJY…VN` adresi Circle'ın *mainnet* USDC ihraççısıdır, testnet'te arzı 0 olan işe yaramaz bir kopyası var. Kullanmayın. Hesap fonlama için Friendbot, USDC için handbook'taki Circle faucet.

## Kontrat (Smart Contracts skill'inden)

- Kurulum: `stellar contract init`, `rustup target add wasm32v1-none`. **`rustup` gerekir**; makinedeki Rust Homebrew kurulumunda `rustup` yok.
- `soroban-sdk` ana sürümü protokol sürümünü izler; ağın protokol sürümüne göre sabitleyin.
- Depolama: instance (ayarlar), persistent (kalıcı veri), temporary. Her kayıt kiralıktır ve arşivlenebilir, `extend_ttl` düzenli çağrılmalı.
- Kimlik: durum değiştiren her fonksiyonda `require_auth()`. Kritik: `overflow-checks = true` (release profili).
- Event'ler tipli struct ve `#[topic]` alanlarıyla yayınlanır (arayüz sponsor güvencesi, eksik ödeme, tahsisat ve iade olaylarını dinler).
- Test: `testutils` ve `env.mock_all_auths()`. Deploy: `stellar contract build`, `stellar contract deploy --wasm ... --source-account <hesap> --network testnet`.
- Kontrol listesi: "Soroban Common Mistakes" skill'i ile deploy öncesi gözden geçirin. Kontrat upgrade edilemez kalacak (plan kararı), bunu README'de yazın.

## Anchor (Standards ve Anchors skill'lerinden)

- Akış: SEP-1 (`stellar.toml`) → SEP-10 (imza ile JWT) → SEP-24 (interaktif) veya SEP-6 (API). SEP-38 fiyat/quote, SEP-12 KYC, SEP-31 sınır ötesi ödeme. Hangi SEP'in kullanılacağı `/info` ve anchor'ın desteğine bağlı.
- **SEP-24 arayüzü popup veya webview'de açılır, iframe'de açılmaz** (anchor `X-Frame-Options: DENY` gönderir). Popup'ı açmadan önce `postMessage` dinleyicisi kaydedilir.
- Tutarlar string'dir (`parseFloat` 7 ondalığı bozar). Varlık kimliği kod + ihraççıdır.
- Durum bir makinedir: `incomplete → pending_user_transfer_start → pending_anchor / pending_external → completed`. Özel durumlar: `pending_trust` (trustline ekle), `pending_user` (arayüzü yeniden aç), `on_hold` (inceleme). Her duruma bir kullanıcı eylemi eşleyin.
- 401'de SEP-10'u sessizce yenileyin, akışı baştan başlatmayın.
- Test için: **testanchor.stellar.org**, Demo Wallet, `@stellar/anchor-tests`. Gerçek TRY anchor'ı workshop'ta netleşecek.

## Plana etkisi
- Trustline kontrolünün `join_pool`'da olması doğrulandı (planda var).
- SEP listesi (1, 6, 10, 12, 24, 31, 38) Anchors skill'inde de geçiyor, plandaki "SEP-24 tercih, kesin değil" yaklaşımı doğru.
- `testanchor.stellar.org` sandbox olarak mevcut. Jüri sandbox'ı yeterli sayar mı sorusu organizatöre sorulmaya devam etmeli.
- Node sürümü: skill Node 22+ öneriyor, makinede v20.19.4 var. Kurulumda doğrulanmalı.
