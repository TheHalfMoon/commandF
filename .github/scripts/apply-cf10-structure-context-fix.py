from pathlib import Path

path = Path('tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java')
text = path.read_text()

def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f'expected exactly one replacement, found {count}: {old[:120]!r}')
    text = text.replace(old, new, 1)

replace_once(
    'import com.fasterxml.jackson.databind.SerializationFeature;\n',
    'import com.fasterxml.jackson.databind.SerializationFeature;\n'
    'import org.hl7.fhir.convertors.loaders.loaderR5.NullLoaderKnowledgeProviderR5;\n'
    'import org.hl7.fhir.convertors.loaders.loaderR5.R4ToR5Loader;\n',
)
replace_once(
    'import org.hl7.fhir.r5.model.StructureDefinition;\n',
    'import org.hl7.fhir.r5.model.StructureDefinition;\n'
    'import org.hl7.fhir.utilities.Utilities;\n'
    'import org.hl7.fhir.utilities.VersionUtilities;\n',
)
replace_once(
    '      IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());\n',
    '      IContextResourceLoader dependencyLoader = structureDefinitionLoader(dependency);\n',
)
replace_once(
    '  private static NpmPackage loadPackage(Path path) throws IOException {\n',
    '''  private static IContextResourceLoader structureDefinitionLoader(NpmPackage dependency) {
    String version = dependency.fhirVersion();
    if (!VersionUtilities.isR4Ver(version)) {
      throw new IllegalArgumentException(
          "oracle dependency context must be FHIR R4 but " + dependency.name() + "#"
              + dependency.version() + " declares " + version);
    }
    return new R4ToR5Loader(
        Utilities.stringSet("StructureDefinition"),
        new NullLoaderKnowledgeProviderR5(),
        version);
  }

  private static NpmPackage loadPackage(Path path) throws IOException {
''',
)

path.write_text(text)
print('minimal StructureDefinition-only dependency context patch applied')
