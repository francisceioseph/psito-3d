// The page is in English in the HTML, and without JavaScript it stays so.
// Portuguese lives here: each [data-t] gets the text of its key, and
// [data-t-alt] the description of its image. Copy buttons are added here too.
const STRINGS = {
  en: { copy: 'Copy', copied: 'Copied', other: 'Português' },
  pt: {
    other: 'English',
    heroTitle: 'Paisagens como nos anos 90.',
    heroLead: 'Arraste o sol pelo céu, sorteie um mundo novo com um dado, derreta esferas cromadas umas nas outras e veja a imagem ficar nítida enquanto você olha. Para Linux e Windows. Sem anúncios, sem telemetria e sem conta.',
    install: 'Instalar',
    sourceCode: 'Código-fonte',
    shot: 'Uma ilha vulcânica num mar calmo, com uma esfera cromada, e os ajustes à direita',
    shotNote: 'A ilha vulcânica vem de um plugin escrito em Lua.',
    whatItDoes: 'O que ele faz',
    f1: 'Um sol que você arrasta num disco do céu, do meio-dia ao pôr do sol e à noite',
    f2: 'Surpreenda-me: um mundo inteiro a partir de uma semente',
    f3: 'Terrenos de formas prontas, de plugins em Lua ou de uma imagem, onde o branco é alto',
    f4: 'O mar infinito, névoa, neblina nos vales, bruma e nuvens',
    f5: 'Esferas, caixas, cones e anéis cromados, de ouro, de vidro ou de argila, somados, cavados ou derretidos',
    f6: 'Filtros de 256 cores ou de tela de tubo, temas, desfazer e imagens salvas em PNG',
    windowsIntro: 'No Windows 10 (1809 ou mais novo) ou no Windows 11, em x64. Não pede administrador.',
    windowsDownload: 'Baixar o instalador',
    windowsNote: 'O SmartScreen pode avisar que o editor é desconhecido, porque o instalador não tem um certificado pago. Escolha Mais informações e depois Executar assim mesmo.',
    debIntro: 'No Debian 13, Ubuntu 24.04, Mint 22 ou mais novos, em amd64. Ele atualiza junto com o sistema.',
    fedoraIntro: 'Em x86_64. Ele atualiza junto com o sistema. A primeira instalação pede para aceitar a chave de assinatura.',
    fromSource: 'A partir do código',
    sourceIntro: 'Em qualquer sistema com Rust e uma GPU com Vulkan, Metal ou DirectX 12, inclusive o macOS.',
    footer: 'O Psito 3D é software livre, sob a Licença Apache 2.0.',
    plugins: 'Escreva um plugin',
    copy: 'Copiar',
    copied: 'Copiado',
  },
};

const english = {};
document.querySelectorAll('[data-t]').forEach((el) => { english[el.dataset.t] = el.textContent; });
document.querySelectorAll('[data-t-alt]').forEach((el) => { english[el.dataset.tAlt] = el.alt; });

function apply(lang) {
  const t = { ...english, ...STRINGS.en, ...(STRINGS[lang] || {}) };
  document.documentElement.lang = lang;
  document.querySelectorAll('[data-t]').forEach((el) => { el.textContent = t[el.dataset.t]; });
  document.querySelectorAll('[data-t-alt]').forEach((el) => { el.alt = t[el.dataset.tAlt]; });
  document.querySelectorAll('.copy').forEach((b) => { b.textContent = t.copy; });
  document.getElementById('lang').textContent = t.other;
  window.currentStrings = t;
}

document.querySelectorAll('.cmd').forEach((box) => {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'copy';
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(box.querySelector('pre').textContent);
      button.textContent = window.currentStrings.copied;
      setTimeout(() => { button.textContent = window.currentStrings.copy; }, 1500);
    } catch {
      window.getSelection().selectAllChildren(box.querySelector('pre'));
    }
  });
  box.appendChild(button);
});

let lang = (navigator.language || 'en').toLowerCase().startsWith('pt') ? 'pt' : 'en';
try { lang = localStorage.getItem('lang') || lang; } catch { /* private window */ }
apply(lang);
document.getElementById('lang').addEventListener('click', () => {
  lang = lang === 'pt' ? 'en' : 'pt';
  try { localStorage.setItem('lang', lang); } catch { /* private window */ }
  apply(lang);
});
