# AI denetimi ve Türkiye'de gerçek ürün yolu

**Durum (19 Eylül 2026):** Bu belge mevzuat taramasına dayalı gelecekteki ürün tasarımıdır; faaliyet izni, BDDK görüşü veya hukuki uygunluk belgesi değildir. Şimdiki karar [Mainnet'te işlemsiz tanıtım](mainnet-showcase.md) ve ayrı Testnet demosudur; lisans başvurusu yapılmayacak, anchor gerçek TRY'ye bağlanmayacaktır. Gerçek müşteri fonu, gerçek ev/araç tahsisatı ve gerçek sözleşme yoktur.

**Kapsam:** Aşağıdaki lisanslı şirket yolu, Fuzul benzeri **halka sunulan ev/araç tasarruf finansmanı** hedefi içindir. Tanıdık kişilerin kendi aralarında kurduğu altın günü her durumda bu şirket modeline girer sonucu çıkarılmamalıdır. [Altın günü ile düzenlenmiş finansman arasındaki sınır](altin-gunu-legal-boundary.md) ayrıca incelenmiştir.

## 1. İki farklı çalışma alanı

| | Hackathon prototipi | Türkiye'de gerçek ürün hedefi |
|---|---|---|
| Fon | Yalnızca Testnet varlığı; Soroban havuz kurallarını uygular | Lisanslı tasarruf finansman şirketinin mevzuata uygun ayrılmış tasarruf fon havuzu; izinli TL araçlar |
| Sözleşme | Katılımcıların demo koşullarını cüzdanla onaylaması | Lisanslı şirketle müşterinin usulüne uygun kurduğu tasarruf finansman sözleşmesi |
| Tahsisat | Test satıcısı cüzdanına Testnet varlığı | Şartları sağlanınca şirketin satıcının banka hesabına ödeme yapması; mülkiyet ve teminat işlemleri |
| AI | Test verisi üzerinde gerekçeli risk raporu önerisi | Yetkili şirketin kontrolünde yardımcı inceleme; karar ve sorumluluk şirketin yetkili personelinde |
| Zincir | Demo fon transferini ve koşulları uygular | İzin verilen kapsamda karar/belge bütünlüğü kaydı; TL fonunun sahibi veya saklayıcısı olduğu varsayılmaz |

Testnet'te çalışan bir kuralın Türkiye'de müşteri fonları için kullanılabilir olduğu sonucu çıkarılamaz. Gerçek ürün için ayrı hukuki ürün, sözleşme ve fon akışı tasarlanır. **Tamamen zincir üstünde USDC havuzu** istenirse, BDDK'nın bu somut yapı için yazılı değerlendirmesi ve gerekli diğer kurum görüşleri alınmadan uygunluk iddia edilmez.

## 2. AI güvenlik denetçisinin sınırları

AI; grup koşulları ile imzalanacak belge sürümleri arasındaki farkları, satıcı bilgisi ve ödeme talimatındaki tutarsızlıkları, mükerrer belgeleri, gecikme/iade kayıtlarını ve olağan dışı işlemleri inceleyebilir. Soroban kodu için yetkilendirme, muhasebe, TTL ve yükseltme yetkisi gibi riskleri geliştiriciye raporlayabilir. Her bulgu ilgili belge/kod sürümüne ve doğrulanabilir kurala dayanır. Modelin önerisi **inceleme gerektirir** durumudur; hukuki geçerlilik veya bağımsız güvenlik denetimi sertifikası değildir.

- AI'a fon cüzdanı, banka ödeme talimatı, kontrat yöneticisi yetkisi verilmez. AI çıktısı tek başına tahsisatı açmaz, sözleşmeyi değiştirmez, iadeyi reddetmez veya süresiz dondurmaz.
- Tutar, yetkili satıcı, sürüm, süre, mükerrer ödeme ve iade yeterliliği gibi kesin şartlar deterministik kurallarla kontrol edilir. Belirsiz belge/kimlik bulgusu yetkili personele sevk edilir; mevzuattaki ödeme ve iade süreleri model yanıtı beklerken kaybolmaz.
- API v12 demosunda doğrulayıcı rolü yoktur. Gerçek üründe lisanslı şirketin yetkili personeli raporu ve dayanak belgeleri inceler. İtiraz ve düzeltme izi tutulur. AI kod incelemesi, testler ve bağımsız akıllı kontrat güvenlik denetimi ayrı kontrollerdir.
- İnceleme kaydı `işlem ID + belge sürümü/özeti + kural sürümü + model sürümü + bulgu gerekçesi + insan kararı + zaman` bağlamını taşır. Gerçek kimlik ve finansal belgeler herkese açık zincire yazılmaz; hash/özetin kendisi de bağlama göre kişisel veri sayılabileceğinden KVKK değerlendirmesi gerekir.
- Harici AI sağlayıcısına belge gönderimi, saklama, sınır ötesi aktarım ve üçüncü taraf erişimi çözülmeden gerçek müşteri verisi işlenmez. Özellikle tasarruf finansmanı, muhasebe/raporlama ve iç sistemlerin üçüncü tarafa devredilememesine ilişkin dış hizmet sınırı incelenir (Yönetmelik md. 32). En düşük riskli başlangıç, lisanslı şirketin kontrol ettiği ortamda yardımcı araçtır.

**Testnet gösterimi:** İmzalanan demo şartlarıyla farklı satıcı/tutar girildiğinde AI gerekçeli uyarı üretebilir; uyarı arayüzde gösterilebilir; çalışan AI entegrasyonu henüz yoktur. Kontratın izin kontrolü AI'ın serbest metin sonucuna dayanmaz. AI entegrasyonu, model ve örnek gerçek işlem kanıtlarıyla çalıştırılmadan ürün özelliği olarak "tamamlandı" işaretlenmez.

## 3. Gerçek sözleşme nasıl kurulabilir?

1. **Yetkili taraf:** Tasarruf finansmanı sözleşmesinin tarafı ve yükümlüsü BDDK'dan faaliyet izni olan şirket olur; teknoloji sağlayıcısı kendini lisanslı şirket veya resmî onay makamı gibi sunmaz. Kendi şirketimizle faaliyet gösterme seçeneği ayrıca kuruluş ve faaliyet izinleri gerektirir (6361 sayılı Kanun md. 7, 39/A). Ortaklık modeli de dış hizmet yasağına göre hukuk ve BDDK ile yapılandırılır.
2. **Kimlik ve irade:** Müşteri kimliği mevzuata uygun tespit edilir. BDDK'nın uzaktan kimlik/elektronik sözleşme yönetmeliği md. 12-13, kimlik doğrulama sonrası internet veya mobil kanalda sözleşme kurulması ve belirli şartlarda yazılı şeklin yerine geçen yöntemleri düzenler. Alternatif güvenli elektronik imza yolu işlem türünün şekil şartlarına göre değerlendirilir. Salt Stellar cüzdan imzası, AI raporu veya zincirdeki hash bu süreçlerin yerine otomatik geçmez.
3. **Sözleşme içeriği ve haklar:** Şirket; katılım/organizasyon ücreti, tasarruf ve finansman planı, teslim koşulları, ödeme aksaması, cayma/fesih ve iadeleri kanun ve yönetmeliğe göre açıkça belirtir. Kanun md. 39/A'daki 14 günlük cayma ve tasarruf dönemi içindeki fesih hakkı ile ilgili iade usulleri AI ya da kontrat kuralıyla bertaraf edilmez. Demo `abort_pool` kuralı gerçek sözleşmedeki fesih prosedürüne aynen kopyalanmaz.
4. **İspat ve kayıt:** Müşteriye gösterilen sözleşmenin tam sürümü, kimlik doğrulama/irade kaydı ve güvenli arşivi şirketin usulüne uygun sisteminde tutulur. İzin verildiği ölçüde zincirde yalnızca bütünlük ve karar izi saklanır. 5070 sayılı Kanun güvenli elektronik imzaya hukuki sonuç tanır; hangi elektronik yöntemin bu ürün için yeterli olduğu işlem bazında teyit edilir.

## 4. Fon ve teslimat için uygulanabilir hedef akış

1. Müşteri katkısı lisanslı şirketin diğer hesaplarından ayrılmış tasarruf fon havuzu düzenine alınır; izin verilen araçlarda değerlendirilir. USDC/Soroban saklaması mevcut listedeki bir araç olarak varsayılmaz (6361 md. 39/B; Yönetmelik md. 23).
2. Şirketin kendi sistemindeki muhasebe ve bankadaki hareketler dönemsel olarak mutabıklaştırılır. Rol ayrımı, sınırlı ödeme yetkisi, ikinci kişi onayı, satıcı hesabı doğrulaması ve bağımsız inceleme amacı dışı transfer riskini azaltır. Zincire kaydedilen rapor tek başına banka transferini teknik olarak durdurmaz; bu nedenle mutlak "kimse kaçamaz" garantisi verilmez.
3. Tahsisat, sözleşme türüne göre asgari tasarruf/süre koşulları ve şirketin doğrulaması sağlanınca satıcının banka hesabına ödenir (Yönetmelik md. 17 ve 21). Demo dört kişilik grubun ilk turda ödeme yapması gerçek ürüne aynen aktarılmaz. Gerekli ipotek/rehin ve tapu/ruhsat işlemleri ayrıca yürütülür (Yönetmelik md. 22).
4. AI satıcı/evrak/işlem uyumsuzluğunu işaretler; nihai onay ve banka talimatı yetkili insan süreçlerinde kalır. Uygun görülürse belge ve karar izleri müşteri tarafından doğrulanabilir; banka hareketi kanıtı ile zincir kaydı eşleştirilmeden "ödendi" gösterilmez.

**Alternatif araştırma:** Gerçek fonun doğrudan akıllı kontratta tutulması veya kripto varlıkla satıcı ödemesi hedeflenirse BDDK'ya somut fon sahipliği, tasfiye, iade ve denetim yapısıyla yazılı görüş sorulur; TCMB'nin kripto ödemeleri düzenlemesi ve gerekiyorsa SPK yetkileri ayrıca incelenir. Olumlu görüş/izin yokken bu model production yol haritasında onaylanmış kabul edilmez.

## 5. Başlamadan önce çözülmesi gereken kararlar

- BDDK lisanslı şirket ve sorumlu sözleşme tarafı kim? AI sağlayıcısı hangi görevleri üstlenebilir; md. 32 açısından dış hizmetin sınırı nedir?
- Kimlik tespiti ve elektronik sözleşme kanalı kim tarafından işletilecek; müşteri hakları ve belge sürümleri nasıl arşivlenecek?
- Hangi bankada hangi ayrılmış hesaplar ve ödeme yetkileri kullanılacak; bağımsız mutabakat ve olay incelemesi kimde?
- İlk tahsisat, kura/sabit sıra, gecikme, varsa hukuki teminat ve iade kuralları gerçek sözleşme türüne uygun mu?
- AI'ın işleyeceği veriler, veri sorumlusu, saklama süresi, üçüncü taraf aktarımı, model hatasına itiraz ve insan incelemesi nasıl düzenlenecek?
- Hackathon DOC'undaki **gerçek TL → kullanılabilir bakiye** şartını karşılayan yetkili anchor/yerel ödeme ortağı var mı? Testnet faucet ve AI raporu bu şartı karşılamaz; gerçek TL akışı lisans ve ödeme/kripto düzenlemeleri netleşmeden havuza bağlanmaz.

## Kaynaklar

- [6361 sayılı Kanun](https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=6361&MevzuatTur=1&MevzuatTertip=5), md. 7, 39/A, 39/B.
- [Tasarruf Finansman Şirketlerinin Kuruluş ve Faaliyet Esasları Hakkında Yönetmelik](https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=38500&MevzuatTur=7&MevzuatTertip=5), md. 17, 21-23, 32.
- [BDDK uzaktan kimlik ve elektronik sözleşme yönetmeliği](https://www.mevzuat.gov.tr/mevzuat?MevzuatNo=39368&MevzuatTur=7&MevzuatTertip=5), md. 12-13; [BDDK'nın ilgili hükümleri aktaran resmî metni](https://www.bddk.org.tr/Mevzuat/DokumanGetir/1171).
- [BTK, güvenli elektronik imzanın hukuki sonucu](https://btk.gov.tr/elektronik-imza-genel-bilgi).
- [TCMB, Ödemelerde Kripto Varlıkların Kullanılmamasına Dair Yönetmelik](https://www.tcmb.gov.tr/wps/wcm/connect/6937855a-7c29-4d08-a26e-51ef3273c022/%C3%96demelerde+Kripto+Varl%C4%B1klar%C4%B1n+Kullan%C4%B1lmamas%C4%B1na+Dair+Y%C3%B6netmelik.pdf?MOD=AJPERES).
- [KVKK, Üretken Yapay Zekâ ve Kişisel Verilerin Korunması Rehberi](https://www.kvkk.gov.tr/Icerik/8547/uretken-yapay-zeka-ve-kisisel-verilerin-korunmasi-rehberi-15-soruda).
- [Rise In x Stellar Hackathon DOC](https://docs.google.com/document/d/1arP0Oih9txU1mOYWhdRHYIg3u4XFBrF3/edit); yerel özet: [hackathon-handbook.md](hackathon-handbook.md).
