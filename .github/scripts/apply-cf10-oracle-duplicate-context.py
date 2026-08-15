from pathlib import Path

path = Path("tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java")
text = path.read_text()
old = '''    NpmPackage side = loadPackage(sidePath);
    for (Path contextPath : contextPaths) {
      NpmPackage dependency = loadPackage(contextPath);
      if (samePackage(core, dependency) || samePackage(side, dependency)) {
        continue;
      }
      IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());
      dependencyLoader.getTypes().retainAll(Set.of("StructureDefinition"));
      context.loadFromPackage(dependency, dependencyLoader, false);
    }
    if (!samePackage(core, side)) {
'''
new = '''    NpmPackage side = loadPackage(sidePath);
    context.setAllowLoadingDuplicates(true);
    try {
      for (Path contextPath : contextPaths) {
        NpmPackage dependency = loadPackage(contextPath);
        if (samePackage(core, dependency) || samePackage(side, dependency)) {
          continue;
        }
        IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());
        dependencyLoader.getTypes().retainAll(Set.of("StructureDefinition"));
        context.loadFromPackage(dependency, dependencyLoader, false);
      }
    } finally {
      context.setAllowLoadingDuplicates(false);
    }
    if (!samePackage(core, side)) {
'''
if text.count(old) != 1:
    raise SystemExit(f"expected one dependency context block, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))
print("oracle dependency duplicate coexistence scope applied")
