"""One-off lossless PROTOCOL topic migration proof against the pre-split commit.

Run from the repository root: python3 tools/verify_protocol_migration.py
Checks content-line multiplicity, old anchors, catalog states and old evidence.
This proves a documentation move, not protocol correctness or acceptance.
"""
from pathlib import Path
from collections import Counter
import hashlib,json,posixpath,re,subprocess,sys
from check_docs import markdown_anchors
# Immutable source of the topic migration, not a current-status source.
base='cbe69c128e299a4fc0bf8bcba358a24fdaeb120c'
before=subprocess.check_output(['git','show',base+':docs/PROTOCOL.md'],text=True)
index=Path('docs/PROTOCOL.md').read_text()
pattern=re.compile(r'\]\((<[^>]+>|[^\s)]+)(?:\s+"[^"]*")?\)')
def canonical(line,source):
 def replace(m):
  value=m[1];target=value.strip('<>')
  if re.match(r'^[a-zA-Z][\w+.-]*:',target):return m[0]
  name,sep,fragment=target.partition('#')
  name=posixpath.normpath(posixpath.join(posixpath.dirname(source),name))if name else source
  if name.startswith('docs/protocol/'):name='docs/PROTOCOL.md'
  target=name+(sep+fragment if sep else '')
  if value.startswith('<'):target='<'+target+'>'
  return m[0].replace(value,target,1)
 return pattern.sub(replace,line)
# Original introductory text and replacement index/navigation/stubs are scaffolding.
# All original content from the first section onwards must survive with multiplicity.
original=Counter(canonical(line,'docs/PROTOCOL.md')for line in before.splitlines()[10:])
a=index.index('## 1. Định danh và nguồn chuẩn');b=index.index('## Tra theo chủ đề',a)
actual=Counter(canonical(line,'docs/PROTOCOL.md')for line in index[a:b].splitlines())
files=sorted(Path('docs/protocol').glob('*.md'));assert 10<=len(files)<=14
for p in files:
 lines=p.read_text().splitlines();assert len(lines)<=1200,p
 assert lines[-1] == '<!-- protocol-topic-content-end -->'
 actual.update(canonical(line,str(p))for line in lines[4:-1])
assert original==actual,{'removed':list((original-actual).items())[:10],'added':list((actual-original).items())[:10]}
assert markdown_anchors(before)<=markdown_anchors(index),sorted(markdown_anchors(before)-markdown_anchors(index))
# Catalog locator migration must not change any non-locator field/state.
old=json.loads(subprocess.check_output(['git','show',base+':docs/features.json'],text=True));current=json.loads(Path('docs/features.json').read_text())
def restore(v):
 if isinstance(v,dict):return{k:restore(x)for k,x in v.items()}
 if isinstance(v,list):return[restore(x)for x in v]
 if isinstance(v,str)and v.startswith('docs/protocol/'):
  return 'docs/PROTOCOL.md#'+v.split('#',1)[1]
 return v
assert old==restore(current)
# Existing evidence/complete records/audits and CORE_DOCS checker remain byte-identical.
changed_evidence = subprocess.check_output(['git','diff',base,'--name-only','--diff-filter=MDRT','--','docs/evidence/'],text=True).splitlines()
# The checkpoint index must append this one new checkpoint/audit; all existing
# records, audits and runtime metadata remain immutable.
assert set(changed_evidence) <= {'docs/evidence/checkpoints/manifest.json'}
assert not subprocess.check_output(['git','diff',base,'--name-only','--','rust/','server/','tools/check_checkpoints.py'],text=True).strip()
result=dict(result='pass',baseline=base,topic_files=len(files),original_content_lines=sum(original.values()),migrated_content_lines=sum(actual.values()),normalized_multiset_sha256=hashlib.sha256(json.dumps(sorted(original.items()),ensure_ascii=False).encode()).hexdigest(),old_anchors_preserved=len(markdown_anchors(before)),index_lines=len(index.splitlines()),index_without_stubs_lines=len(index.split('<!-- protocol-compatibility-stubs -->')[0].splitlines()),max_topic_lines=max(len(p.read_text().splitlines())for p in files),catalog_states_unchanged=True,old_evidence_unchanged=True,runtime_and_checkpoint_checker_unchanged=True,normalization='Only relative Markdown link paths; original intro/new index/navigation/stubs excluded. Multiplicity and every other content character preserved.')
print(json.dumps(result,indent=2))
