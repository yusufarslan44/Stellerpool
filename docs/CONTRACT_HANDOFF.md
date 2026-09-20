# Kontrat ekibi görev listesi — API v10 (kura + 30 üye)

**Durum (19 Eylül 2026, gece):** Faz 13 tamamlandı — bu belgedeki iş uygulandı. Testnet'te **kura + 30 üye destekleyen kontrat (API v10)** yayında: `CC7W3SKQHBLZ2JPTGSK42H6IAJQ22A4PUK6CSN2T4PUJRY4LQ445GYMB` ([README](../README.md#contract-and-deployment-proof-testnet)). `stellar contract info interface` ve `stellar contract bindings typescript` çıktısı `frontend/src/services/pool.ts` / `types/pool.ts` ile alan alan doğrulandı: `create_pool`'a `order_mode` (member_limit'ten sonra), `Pool.order_mode`, `RoundState.recipient: Option<Address>`, `RoundPhase::AwaitingDraw`, `draw_recipient(caller, pool_id) -> Address`. `frontend/.env`'e `VITE_MAX_MEMBERS=30` yazıldı (yerel, gitignore'lu). Aşağıdaki metin görev tanımı olarak olduğu gibi bırakıldı.

**Not:** `README.md`'nin "Kontrat ve dağıtım kanıtı" bölümü bu kontrat ID'siyle henüz güncellenmedi — bu oturumda README.md'ye dokunulmaması ayrıca istendi; bir sonraki README güncellemesinde eklenmeli.

---

Bu belge **yalnızca yeni işi** tanımlar: Fuzul Ev/Oto mantığına yaklaşmak için **kura modu** ve **12 → 30 üye**. Arayüz ikisine de hazırdır, kontrat gelene kadar kapalı kalır:
- Kontratta `draw_recipient` yoksa formda "Kura" seçeneği kapanır; `create_pool`'a `order_mode` hiç gönderilmez.
- Üye üst sınırı `VITE_MAX_MEMBERS` ile açılır (varsayılan 12). Kontrat 30'u destekleyince `frontend/.env` içine `VITE_MAX_MEMBERS=30` yazılır.
- Eski sponsorlu (v8) kontrat algılanırsa (`fund_guarantee`/`top_up` var) arayüz işlem göndermez.

Parametre/alan adları `stellar contract bindings typescript` ile karşılaştırılmalıdır; fark olursa yalnızca `frontend/src/services/pool.ts` ve `types/pool.ts` değişir.

## 1. Kura modu

**Tipler:** `enum OrderMode { Fixed, Draw }`; `RoundPhase` içine `AwaitingDraw` (`Collecting → [Grace] → AwaitingDraw → AwaitingPurchase → Settled`); `Pool.order_mode`; `RoundState.recipient: Option<Address>`. `create_pool` parametre listesine `order_mode` eklenir (`member_limit`'ten sonra); v9'daki diğer parametreler aynı kalır.

**Kurallar**
1. `Fixed`: bugünkü davranış. `propose_terms` tam sıra ister, alıcı tur başında bellidir, tüm katkılar gelince doğrudan `AwaitingPurchase`.
2. `Draw`: `propose_terms` **boş `recipient_order`** ister; yalnızca doğrulayıcılar önerilir. Tur başında `recipient = None`.
3. Draw'da herkes katkısını yatırınca faz **`AwaitingDraw`** olur, alım süresi (`purchase_deadline`) burada başlar.
4. **`draw_recipient(pool_id, caller) -> Address`**: yalnızca `AwaitingDraw`'da, herkes çağırabilir (özel yetki yok). Aday kümesi `received == false` üyelerdir. Kazanan `recipient` olur, `RecipientDrawn` olayı yayınlanır, faz `AwaitingPurchase` olur. Kazanan zaten kendi payını yatırmıştır. Tek aday kalınca deterministik.
5. Rastgelelik `env.prng()` ile yapılır. **Hackathon düzeyidir**: Soroban PRNG'sinin karşı-taraf etkisi resmi dokümandan doğrulanıp belgeye yazılmalı. Gerçek kullanım için commit-reveal veya harici rastgelelik gerekir (kapsam dışı).
6. `purchase_deadline` `AwaitingDraw`'da da geçerlidir: süre dolarsa `abort_pool` ve `claim_refund` mevcut tur katkılarını iade eder.
7. `execute_round`: alıcı `received = true` olur; Draw'da sonraki tur yine `recipient = None` ile başlar.

## 2. 30 üyeye ölçekleme

`MAX_MEMBERS` 12 → **30** (`lib.rs` satır ~18). Tek işlemde üye sayısı kadar depolama okuması/yazması yapan yerler:
- `execute_round`: iki üye döngüsü (her üyenin `RefundLiability` girdisi okunur/sıfırlanır). Çözüm: iade hakkını `has_deposit(havuz, tur, üye) && !refund_claimed` olarak turdan türetin, sıfırlama döngüsünü kaldırın; `paid_count` sayacı kullanın.
- `start_pool`: `has_terms_approval` döngüsü yerine `terms_approvals.len() == member_limit` (tekrarsız eklendiği garanti edilerek).
- `verifiers` ≤ 10 kalır; `RoundState.paid`/`Pool.members` `Vec<Address>` olarak kalabilir.

**Kabul ölçütü:** 30 üyeli havuzda `start_pool`, `deposit`, `draw_recipient`, `execute_round`, `claim_refund` Testnet kaynak simülasyonunda **okuma/yazma girdisi ve CPU limitlerinin altında** kalır. Simülasyon çıktısını belgeye ekleyin.

## 3. Testler (`cargo test`)

- Draw: 4 ve **30 üyeyle** tam akış (`Address::generate`).
- Draw: kazanan yalnızca `received == false` üyelerden çıkar; teslim almış üye bir daha çıkmaz; son adayda deterministik.
- Draw: ödenmemiş turda `draw_recipient` reddedilir; aynı turda ikinci çekiliş reddedilir; Fixed havuzda reddedilir.
- Draw: `AwaitingDraw` süresi dolunca `abort_pool` → `claim_refund` yalnızca mevcut turu iade eder, çift iade yok.
- Draw'da dolu `recipient_order` reddedilir; Fixed'de eksik/tekrarlı sıra reddedilir.
- Plan.md §5'in kanonik senaryosu Draw için de: erken kura kazananın sonraki turda ödememesi → yalnızca o turun katkıları iade, önceki turlar edilmez.

## 4. Yayın ve devir

1. `scripts/deploy_testnet.sh` ile yeni örnek yayınlayın (yükseltme yok; yeni ID). Sürüm `10` olsun.
2. Yeni kontrat ID'sini ve işlem bağlantılarını `README.md` "Contract and deployment proof" bölümüne yazın. Arayüz için `frontend/.env`: `VITE_ROTATING_POOL_CONTRACT_ID` ve `VITE_MAX_MEMBERS=30`.
3. Bindings çıktısındaki metot/alan adlarını arayüzle karşılaştırın.
4. Eski örnekler (üç sponsorlu/sponsorsuz örnek) canlı kalır; teslim materyalinde yalnızca güncel olan anılmalı.

## 5. Bilinen sınırlar (arayüz ve belgeler bunları açıkça yazar)

- Erken kura kazanan sonraki katkıyı bırakırsa diğerlerinin önceki tur ödemeleri geri alınamaz; grubu büyütmek bunu çözmez. Fuzul/Eminevim benzeri ürünler bunu ipotek/rehin ve şirket taahhüdüyle kapatır, bu prototipte karşılığı yoktur.
- Kura + halka açık büyük grup + ev/araç vaadi, [hukuki sınır belgesinde](altin-gunu-legal-boundary.md) 6361'e yaklaşan unsurlar olarak işaretlidir. Yalnızca Testnet, gerçek para ve teslimat olmadan.
- Zincir üstü rastgelelik hackathon düzeyindedir.
