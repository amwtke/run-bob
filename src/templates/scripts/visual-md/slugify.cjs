const STOP_WORDS = new Set([
  'a','an','the','of','to','for','in','on','at','by','with','and','or','but',
  'is','are','was','were','be','been','being','have','has','had','do','does','did',
  'this','that','these','those','it','its','as','from','about','into','through',
  'one','two','make','create','build','add','remove','outline'
]);

function slugify(prompt) {
  if (!prompt || !prompt.trim()) return `untitled-${Date.now()}`;

  const cleaned = prompt.toLowerCase().replace(/[^a-z0-9\s-]/g, ' ');
  const tokens = cleaned.split(/\s+/).filter(t => t && !STOP_WORDS.has(t));

  if (tokens.length === 0) return `untitled-${Date.now()}`;

  const keywords = tokens.slice(0, 5);
  return keywords.join('-').replace(/^-+|-+$/g, '');
}

if (require.main === module) {
  const input = process.argv.slice(2).join(' ');
  process.stdout.write(slugify(input));
}

module.exports = { slugify };
