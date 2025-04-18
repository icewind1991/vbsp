use crate::*;
use binrw::io::Cursor;
use binrw::BinReaderExt;
use std::borrow::Cow;

pub struct BspFile<'a> {
    data: &'a [u8],
    directories: Directories,
    header: Header,
}

impl<'a> BspFile<'a> {
    pub fn new(data: &'a [u8]) -> BspResult<Self> {
        let mut cursor = Cursor::new(data);
        let header: Header = cursor.read_le().map_err(|err| match err {
            binrw::Error::BadMagic { .. } => {
                BspError::UnexpectedHeader(data[..4].try_into().expect(
                    "always enough data because otherwise a different binrw error would be hit",
                ))
            }
            error => BspError::MalformedData(error),
        })?;
        let mut directories: Directories = cursor.read_le()?;
        if header.version == BspVersion::Version21 && directories.is_l4d2_lump_order(data.len()) {
            directories.fixup_lumps();
        }

        Ok(BspFile {
            data,
            directories,
            header,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn lump_reader(&self, lump: LumpType) -> BspResult<LumpReader<Cursor<Cow<[u8]>>>> {
        let entry = self.get_lump_entry(lump);
        let data = self.get_lump(entry)?;
        Ok(LumpReader::new(data, lump, entry.version))
    }

    pub fn get_lump_entry(&self, lump: LumpType) -> &LumpEntry {
        &self.directories[lump]
    }

    pub fn get_lump(&self, lump: &LumpEntry) -> BspResult<Cow<[u8]>> {
        let raw_data = self
            .data
            .get(lump.offset as usize..lump.offset as usize + lump.length as usize)
            .ok_or(BspError::LumpOutOfBounds(*lump))?;

        Ok(match lump.ident {
            0 => Cow::Borrowed(raw_data),
            _ => {
                let data = lzma_decompress_with_header(raw_data, lump.ident as usize)?;
                Cow::Owned(data)
            }
        })
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LumpType {
    Entities,
    Planes,
    TextureData,
    Vertices,
    Visibility,
    Nodes,
    TextureInfo,
    Faces,
    Lighting,
    Occlusion,
    Leaves,
    FaceIds,
    Edges,
    SurfaceEdges,
    Models,
    WorldLights,
    LeafFaces,
    LeafBrushes,
    Brushes,
    BrushSides,
    Areas,
    AreaPortals,
    Unused0,
    Unused1,
    Unused2,
    Unused3,
    DisplacementInfo,
    OriginalFaces,
    PhysDisplacement,
    PhysCollide,
    VertNormals,
    VertNormalIndices,
    DisplacementLightMapAlphas,
    DisplacementVertices,
    DisplacementLightMapSamplePositions,
    GameLump,
    LeafWaterData,
    Primitives,
    PrimVertices,
    PrimIndices,
    PakFile,
    ClipPortalVertices,
    CubeMaps,
    TextureDataStringData,
    TextureDataStringTable,
    Overlays,
    LeafMinimumDistanceToWater,
    FaceMacroTextureInfo,
    DisplacementTris,
    PhysicsCollideSurface,
    WaterOverlays,
    LeafAmbientIndexHdr,
    LeafAmbientIndex,
    LightingHdr,
    WorldLightsHdr,
    LeafAmbientLightingHdr,
    LeafAmbientLighting,
    XZipPakFile,
    FacesHdr,
    MapFlags,
    OverlayFades,
    OverlaySystemLevels,
    PhysLevel,
    DisplacementMultiBlend,
}

static_assertions::const_assert_eq!(LumpType::DisplacementMultiBlend as usize, 63);
