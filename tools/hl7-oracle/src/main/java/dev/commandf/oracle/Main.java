package dev.commandf.oracle;

import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import org.hl7.fhir.r5.comparison.CanonicalResourceComparer.ChangeAnalysisState;
import org.hl7.fhir.r5.comparison.ComparisonSession;
import org.hl7.fhir.r5.comparison.ResourceComparer;
import org.hl7.fhir.r5.comparison.ResourceComparer.ResourceComparison;
import org.hl7.fhir.r5.comparison.StructuralMatch;
import org.hl7.fhir.r5.comparison.StructureDefinitionComparer.ProfileComparison;
import org.hl7.fhir.r5.context.IContextResourceLoader;
import org.hl7.fhir.r5.context.SimpleWorkerContext;
import org.hl7.fhir.r5.model.StructureDefinition;
import org.hl7.fhir.utilities.i18n.RenderingI18nContext;
import org.hl7.fhir.utilities.npm.NpmPackage;
import org.hl7.fhir.utilities.validation.ValidationMessage;
import org.hl7.fhir.validation.ValidatorUtils;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.TreeSet;

public final class Main {
  static final int SCHEMA = 1;
  static final String ORACLE_PROJECT = "hapifhir/org.hl7.fhir.core";
  static final String ORACLE_RELEASE = "6.10.2";
  static final String ORACLE_SOURCE_COMMIT = "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b";
  static final String CORE_PACKAGE_NAME = "hl7.fhir.r4.core";
  static final String CORE_PACKAGE_VERSION = "4.0.1";

  private static final ObjectMapper JSON = new ObjectMapper()
      .enable(SerializationFeature.INDENT_OUTPUT)
      .enable(SerializationFeature.ORDER_MAP_ENTRIES_BY_KEYS)
      .enable(MapperFeature.SORT_PROPERTIES_ALPHABETICALLY);

  private Main() {
  }

  public static void main(String[] args) {
    try {
      Arguments parsed = Arguments.parse(args);
      Hl7OracleReport report = compare(parsed);
      System.out.write(JSON.writeValueAsBytes(report));
      System.out.write('\n');
    } catch (Exception error) {
      System.err.println("commandf-hl7-oracle: " + safeMessage(error));
      System.exit(1);
    }
  }

  static Hl7OracleReport compare(Arguments args) throws Exception {
    ContextAndPackage left = loadContext(
        args.corePackage(), args.leftContextPackages(), args.leftPackage());
    ContextAndPackage right = loadContext(
        args.corePackage(), args.rightContextPackages(), args.rightPackage());

    StructureDefinition leftResource = fetchStructureDefinition(
        left.context(), args.leftUrl(), args.leftVersion(), "left");
    StructureDefinition rightResource = fetchStructureDefinition(
        right.context(), args.rightUrl(), args.rightVersion(), "right");

    ComparisonSession session = new ComparisonSession(
        new RenderingI18nContext(),
        left.context(),
        right.context(),
        "commandF HL7 oracle",
        null,
        null);
    session.setAnnotate(false);

    ResourceComparison comparison = session.compare(leftResource, rightResource);
    if (!(comparison instanceof ProfileComparison profile)) {
      String actual = comparison == null ? "null" : comparison.getClass().getName();
      if (comparison instanceof ResourceComparer.PlaceHolderComparison placeholder
          && placeholder.getE() != null) {
        Throwable cause = placeholder.getE();
        throw new IllegalStateException(
            "HL7 comparison failed inside " + actual + ": "
                + cause.getClass().getName() + ": " + safeMessage(cause));
      }
      throw new IllegalStateException("HL7 comparison did not return ProfileComparison: " + actual);
    }

    TreeSet<OracleMessage> normalizedMessages = new TreeSet<>(Comparator
        .comparing(OracleMessage::level)
        .thenComparing(OracleMessage::location)
        .thenComparing(OracleMessage::message));

    collectMessages(profile.getMessages(), normalizedMessages);
    for (Map.Entry<String, StructuralMatch<String>> entry : profile.getMetadata().entrySet()) {
      collectStructuralMatch(entry.getValue(), normalizedMessages);
    }
    collectStructuralMatch(profile.getCombined(), normalizedMessages);

    OracleStates states = new OracleStates(
        normalizeState(profile.getChangedMetadata()),
        normalizeState(profile.getChangedDefinitions()),
        normalizeState(profile.getChangedContent()),
        normalizeState(profile.getChangedContentInterpretation()));

    return new Hl7OracleReport(
        SCHEMA,
        new OracleIdentity(ORACLE_PROJECT, ORACLE_RELEASE, ORACLE_SOURCE_COMMIT),
        resourceIdentity(leftResource),
        resourceIdentity(rightResource),
        states,
        List.copyOf(normalizedMessages));
  }

  private static ContextAndPackage loadContext(
      Path corePath, List<Path> contextPaths, Path sidePath) throws Exception {
    NpmPackage core = loadPackage(corePath);
    requirePackage(core, CORE_PACKAGE_NAME, CORE_PACKAGE_VERSION, "core");

    IContextResourceLoader coreLoader = ValidatorUtils.loaderForVersion(core.fhirVersion());
    SimpleWorkerContext context = new SimpleWorkerContext.SimpleWorkerContextBuilder()
        .withAllowLoadingDuplicates(true)
        .fromPackage(core, coreLoader, false);
    context.setAllowLoadingDuplicates(false);
    context.setCanRunWithoutTerminology(true);

    NpmPackage side = loadPackage(sidePath);
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
      IContextResourceLoader sideLoader = ValidatorUtils.loaderForVersion(side.fhirVersion());
      context.loadFromPackage(side, sideLoader, false);
    }
    return new ContextAndPackage(context, side.name(), side.version());
  }

  private static NpmPackage loadPackage(Path path) throws IOException {
    if (!Files.isRegularFile(path)) {
      throw new IOException("package path is not a regular file: " + path);
    }
    try (InputStream input = Files.newInputStream(path)) {
      return NpmPackage.fromPackage(input, path.getFileName().toString(), false);
    }
  }

  private static boolean samePackage(NpmPackage left, NpmPackage right) {
    return Objects.equals(left.name(), right.name()) && Objects.equals(left.version(), right.version());
  }

  private static void requirePackage(NpmPackage npm, String name, String version, String role) {
    if (!Objects.equals(npm.name(), name) || !Objects.equals(npm.version(), version)) {
      throw new IllegalArgumentException(
          role + " package must be " + name + "#" + version + " but was " + npm.name() + "#" + npm.version());
    }
  }

  private static StructureDefinition fetchStructureDefinition(
      SimpleWorkerContext context,
      String url,
      String version,
      String side) {
    String canonical = version == null || version.isBlank() ? url : url + "|" + version;
    StructureDefinition resource = context.fetchResource(StructureDefinition.class, canonical);
    if (resource == null && version != null && !version.isBlank()) {
      resource = context.fetchResource(StructureDefinition.class, url);
    }
    if (resource == null) {
      throw new IllegalArgumentException("unable to resolve " + side + " StructureDefinition " + canonical);
    }
    return resource;
  }

  private static OracleResourceIdentity resourceIdentity(StructureDefinition resource) {
    return new OracleResourceIdentity(
        emptyToNull(resource.getUrl()),
        emptyToNull(resource.getVersion()),
        emptyToNull(resource.getId()),
        emptyToNull(resource.getType()));
  }

  private static void collectStructuralMatch(StructuralMatch<?> match, TreeSet<OracleMessage> output) {
    if (match == null) {
      return;
    }
    collectMessages(match.getMessages(), output);
    for (StructuralMatch<?> child : match.getChildren()) {
      collectStructuralMatch(child, output);
    }
  }

  private static void collectMessages(List<ValidationMessage> messages, TreeSet<OracleMessage> output) {
    for (ValidationMessage message : messages) {
      output.add(new OracleMessage(
          message.getLevel().name().toLowerCase(Locale.ROOT),
          nullToEmpty(message.getLocation()),
          nullToEmpty(message.getMessage())));
    }
  }

  private static String normalizeState(ChangeAnalysisState state) {
    return switch (state) {
      case Unknown -> "unknown";
      case NotChanged -> "not_changed";
      case Changed -> "changed";
      case CannotEvaluate -> "cannot_evaluate";
    };
  }

  private static String safeMessage(Throwable error) {
    String message = error.getMessage();
    return message == null || message.isBlank() ? error.getClass().getSimpleName() : message;
  }

  private static String nullToEmpty(String value) {
    return value == null ? "" : value;
  }

  private static String emptyToNull(String value) {
    return value == null || value.isBlank() ? null : value;
  }

  record ContextAndPackage(SimpleWorkerContext context, String packageName, String packageVersion) {
  }

  record OracleIdentity(String project, String release, String source_commit) {
  }

  record OracleResourceIdentity(String url, String version, String id, String type) {
  }

  record OracleStates(String metadata, String definitions, String content, String content_interpretation) {
  }

  record OracleMessage(String level, String location, String message) {
  }

  record Hl7OracleReport(
      int schema,
      OracleIdentity oracle,
      OracleResourceIdentity left,
      OracleResourceIdentity right,
      OracleStates states,
      List<OracleMessage> messages) {
  }

  record Arguments(
      Path corePackage,
      Path leftPackage,
      Path rightPackage,
      List<Path> leftContextPackages,
      List<Path> rightContextPackages,
      String leftUrl,
      String leftVersion,
      String rightUrl,
      String rightVersion) {

    static Arguments parse(String[] args) {
      Map<String, String> values = new LinkedHashMap<>();
      List<Path> leftContextPackages = new ArrayList<>();
      List<Path> rightContextPackages = new ArrayList<>();
      for (int index = 0; index < args.length; index += 2) {
        if (index + 1 >= args.length) {
          throw new IllegalArgumentException("missing value for " + args[index]);
        }
        String key = args[index];
        if (!key.startsWith("--")) {
          throw new IllegalArgumentException("unexpected positional argument: " + key);
        }
        String value = args[index + 1];
        if (key.equals("--left-context-package")) {
          leftContextPackages.add(Path.of(value));
        } else if (key.equals("--right-context-package")) {
          rightContextPackages.add(Path.of(value));
        } else if (values.put(key, value) != null) {
          throw new IllegalArgumentException("duplicate argument: " + key);
        }
      }

      List<String> allowed = List.of(
          "--core-package",
          "--left-package",
          "--right-package",
          "--left-url",
          "--left-version",
          "--right-url",
          "--right-version");
      List<String> unknown = new ArrayList<>();
      for (String key : values.keySet()) {
        if (!allowed.contains(key)) {
          unknown.add(key);
        }
      }
      if (!unknown.isEmpty()) {
        throw new IllegalArgumentException("unknown arguments: " + String.join(", ", unknown));
      }

      return new Arguments(
          Path.of(required(values, "--core-package")),
          Path.of(required(values, "--left-package")),
          Path.of(required(values, "--right-package")),
          List.copyOf(leftContextPackages),
          List.copyOf(rightContextPackages),
          required(values, "--left-url"),
          values.get("--left-version"),
          required(values, "--right-url"),
          values.get("--right-version"));
    }

    private static String required(Map<String, String> values, String key) {
      String value = values.get(key);
      if (value == null || value.isBlank()) {
        throw new IllegalArgumentException("missing required argument " + key);
      }
      return value;
    }
  }
}
