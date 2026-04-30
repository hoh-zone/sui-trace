-- Seed a small set of well-known Sui protocols so the TVL poller has work
-- to do on a fresh install. Operators are expected to extend this through
-- the admin console (or by inserting rows manually).

INSERT INTO protocols (id, name, package_ids, category, website, defillama_slug)
VALUES
  ('cetus-amm',  'Cetus AMM',     '{}', 'dex',     'https://cetus.zone',     'cetus-amm'),
  ('navi',       'Navi Protocol', '{}', 'lending', 'https://www.naviprotocol.io', 'navi-protocol'),
  ('scallop',    'Scallop Lend',  '{}', 'lending', 'https://www.scallop.io', 'scallop-lend'),
  ('suilend',    'Suilend',       '{}', 'lending', 'https://suilend.fi',     'suilend'),
  ('aftermath',  'Aftermath Finance', '{}', 'dex', 'https://aftermath.finance', 'aftermath-amm')
ON CONFLICT (id) DO NOTHING;
